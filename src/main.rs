// pipsqueak - borderless picture-in-picture window mirroring a Wayland monitor.
// Copyright (C) 2026 syndias
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

mod capture;

use std::num::NonZeroU32;
use std::os::unix::process::CommandExt;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::platform::wayland::WindowAttributesExtWayland;
use winit::platform::x11::{WindowAttributesExtX11, WindowType};
use winit::window::{Window, WindowId};

use capture::CapturedFrame;

/// Wakes the event loop when the capture thread has a new frame ready.
#[derive(Debug)]
pub enum UserEvent {
    FrameReady,
}

/// Discrete 16:9 sizes, stepped through with `+`/`-` instead of drag-resize.
/// The window is always non-resizable (min == max) at whatever the current
/// preset is - that's what makes tiling WMs (sway/hyprland/Krohnkite, all
/// confirmed by testing) auto-float it. A window that's *ever* freely
/// resizable loses that; there's no Wayland protocol for a client to
/// declare "float me" independent of that signal.
const SIZE_PRESETS: &[(f64, f64)] = &[
    (320.0, 180.0),
    (480.0, 270.0),
    (640.0, 360.0),
    (800.0, 450.0),
    (960.0, 540.0),
];
const DEFAULT_PRESET: usize = 1; // 480x270

fn apply_preset(window: &Window, index: usize) {
    let (w, h) = SIZE_PRESETS[index];
    let size = LogicalSize::new(w, h);
    window.set_min_inner_size(Some(size));
    window.set_max_inner_size(Some(size));
    let _ = window.request_inner_size(size);
}

struct App {
    window: Option<Rc<Window>>,
    // Kept alive alongside the surface; softbuffer needs the context to outlive it.
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    shared_frame: Arc<Mutex<Option<CapturedFrame>>>,
    /// Signals the running capture thread to stop. `None` means no capture
    /// is currently running.
    capture_cancel: Option<Arc<AtomicBool>>,
    capture_handle: Option<thread::JoinHandle<()>>,
    proxy: EventLoopProxy<UserEvent>,
    preset_index: usize,
}

impl App {
    fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        Self {
            window: None,
            context: None,
            surface: None,
            shared_frame: Arc::new(Mutex::new(None)),
            capture_cancel: None,
            capture_handle: None,
            proxy,
            preset_index: DEFAULT_PRESET,
        }
    }

    fn start_capture(&mut self) {
        let cancel = Arc::new(AtomicBool::new(false));
        let handle = capture::spawn(self.shared_frame.clone(), self.proxy.clone(), cancel.clone());
        self.capture_cancel = Some(cancel);
        self.capture_handle = Some(handle);
    }

    /// Switches monitor by re-executing the whole process instead of
    /// restarting capture in-process. Empirically, xdg-desktop-portal-kde's
    /// ScreenCast backend doesn't handle a second session from the same
    /// app/D-Bus connection - the portal itself logs
    /// "PipeWire remote error: -2 target not found" and the picker never
    /// reappears, even though our own teardown (cancel -> pipewire loop
    /// exits -> session.close()) completes correctly. A fresh process gets
    /// a fresh D-Bus connection, sidestepping whatever that is. Brief
    /// visible flash as the window recreates; acceptable trade-off vs.
    /// chasing a portal-backend bug.
    fn relaunch_for_switch(&mut self) {
        if let Some(cancel) = self.capture_cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        // Block briefly so the current generation's session.close() actually
        // runs - exec() below replaces the process image outright, which
        // abandons any other threads without running their cleanup.
        if let Some(handle) = self.capture_handle.take() {
            let _ = handle.join();
        }
        capture::forget_restore_token();

        let exe = std::env::current_exe().expect("failed to get current exe path");
        let err = std::process::Command::new(exe).exec();
        eprintln!("failed to re-exec for monitor switch: {err}");
    }

    fn redraw(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else {
            return;
        };
        let size = window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return; // window minimized / zero-sized
        };

        surface.resize(width, height).expect("failed to resize surface");
        let mut buffer = surface.buffer_mut().expect("failed to get buffer");

        let frame = self.shared_frame.lock().unwrap();
        match frame.as_ref() {
            Some(frame) if frame.width > 0 && frame.height > 0 => {
                blit_scaled(&mut buffer, size.width, size.height, frame);
            }
            // No frame yet (still waiting on the portal picker / stream).
            _ => buffer.fill(0xFF20_2020),
        }
        drop(frame);

        buffer.present().expect("failed to present buffer");
    }
}

/// Nearest-neighbor scales `frame` to fit inside a `dst_w`x`dst_h` buffer,
/// preserving aspect ratio and letterboxing the rest.
fn blit_scaled(dst: &mut [u32], dst_w: u32, dst_h: u32, frame: &CapturedFrame) {
    let scale = f64::min(
        dst_w as f64 / frame.width as f64,
        dst_h as f64 / frame.height as f64,
    );
    let draw_w = ((frame.width as f64) * scale).round().max(1.0) as u32;
    let draw_h = ((frame.height as f64) * scale).round().max(1.0) as u32;
    let off_x = (dst_w - draw_w) / 2;
    let off_y = (dst_h - draw_h) / 2;

    dst.fill(0xFF10_1010); // letterbox background

    for dy in 0..draw_h {
        let sy = (((dy as f64) / scale) as u32).min(frame.height - 1);
        let dst_row = ((off_y + dy) * dst_w + off_x) as usize;
        let src_row = (sy * frame.width) as usize;
        for dx in 0..draw_w {
            let sx = (((dx as f64) / scale) as u32).min(frame.width - 1);
            dst[dst_row + dx as usize] = frame.pixels[src_row + sx as usize];
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Fixed size (min == max) at whatever the current preset is - see
        // SIZE_PRESETS for why. `+`/`-` step through presets instead of
        // drag-resize.
        let (w, h) = SIZE_PRESETS[self.preset_index];
        let size = LogicalSize::new(w, h);
        let attrs = Window::default_attributes()
            .with_title("pipsqueak")
            .with_decorations(false)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_max_inner_size(size)
            .with_x11_window_type(vec![WindowType::Utility]);
        // `with_name` exists on both the Wayland and X11 ext traits; disambiguate.
        let attrs = WindowAttributesExtWayland::with_name(attrs, "pipsqueak", "pipsqueak");

        let window = Rc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );
        let context = Context::new(window.clone()).expect("failed to create softbuffer context");
        let surface =
            Surface::new(&context, window.clone()).expect("failed to create softbuffer surface");

        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.redraw();

        if self.capture_cancel.is_none() {
            self.start_capture();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // Owned clone (cheap - just an Rc bump), not a borrow of `self.window`,
        // so match arms below are free to call `&mut self` methods like
        // `relaunch_for_switch` while still holding `window`.
        let Some(window) = self.window.clone() else {
            return;
        };
        if window.id() != id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Err(err) = window.drag_window() {
                    eprintln!("drag_window failed: {err}");
                }
            }
            WindowEvent::Resized(_) => self.redraw(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key,
                        ..
                    },
                ..
            } => match logical_key.as_ref() {
                // The logical (produced-character) key, not the physical
                // one - so this is "+"/"-" wherever the layout puts them,
                // not wherever US QWERTY does.
                Key::Character("+" | "=") if self.preset_index + 1 < SIZE_PRESETS.len() => {
                    self.preset_index += 1;
                    apply_preset(&window, self.preset_index);
                }
                Key::Character("-") if self.preset_index > 0 => {
                    self.preset_index -= 1;
                    apply_preset(&window, self.preset_index);
                }
                Key::Named(NamedKey::Tab) => self.relaunch_for_switch(),
                _ => {}
            },
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::FrameReady => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    let proxy = event_loop.create_proxy();
    let mut app = App::new(proxy);
    event_loop.run_app(&mut app).expect("event loop error");
}
