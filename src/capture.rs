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

//! Monitor capture via the xdg-desktop-portal ScreenCast portal + PipeWire.
//!
//! Flow: ask the portal for a monitor (user picks one in a compositor
//! dialog) -> get back a PipeWire node id + fd -> connect a PipeWire stream
//! to that node -> decode raw video frames into `CapturedFrame`s that the
//! renderer can blit directly.

use std::fs;
use std::io::Cursor;
use std::os::fd::OwnedFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ashpd::desktop::screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType};
use ashpd::desktop::PersistMode;
use enumflags2::BitFlags;
use pipewire as pw;
use pw::context::ContextBox;
use pw::loop_::Timeout;
use pw::main_loop::MainLoopBox;
use pw::properties::properties;
use pw::spa;
use pw::stream::StreamBox;
use spa::param::format_utils;
use spa::param::video::{VideoFormat, VideoInfoRaw};
use spa::utils::{Direction, Fraction, Rectangle, SpaTypes};
use winit::event_loop::EventLoopProxy;

use crate::UserEvent;

/// Where the restore token is persisted across runs (`~/.local/state/pipsqueak/`
/// or `$XDG_STATE_HOME/pipsqueak/`, and the equivalent per-app path Flatpak
/// redirects `$HOME` to when sandboxed) so the portal's monitor picker
/// doesn't pop up every single run.
fn restore_token_path() -> Option<std::path::PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "pipsqueak")?;
    Some(dirs.state_dir()?.join("restore_token"))
}

pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    /// One u32 per pixel, already in the 0xAARRGGBB layout `softbuffer`
    /// expects - callers can blit this directly.
    pub pixels: Vec<u32>,
}

/// Deletes the saved restore token so the next capture forces a fresh
/// portal picker instead of silently reusing the last monitor.
pub fn forget_restore_token() {
    if let Some(path) = restore_token_path() {
        let _ = fs::remove_file(path);
    }
}

/// Spawns a background thread that does the portal handshake, then runs the
/// PipeWire stream's loop until `cancel` is set (checked every ~200ms;
/// `MainLoopBox`/friends wrap raw pointers and aren't `Send`, so unlike
/// `shared_frame`/`proxy` there's no way to reach in and stop them from
/// another thread - this is why it's a poll loop instead of a blocking
/// `mainloop.run()`). Frames are written into `shared_frame` and `proxy`
/// wakes the winit event loop so it redraws. On cancel, the stream/session
/// are torn down normally (by simply going out of scope), which is what
/// lets a monitor switch not leak the old screen-share session.
pub fn spawn(
    shared_frame: Arc<Mutex<Option<CapturedFrame>>>,
    proxy: EventLoopProxy<UserEvent>,
    cancel: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(err) => {
                eprintln!("capture: failed to start async runtime: {err}");
                return;
            }
        };

        let (node_id, fd, session) = match rt.block_on(open_portal()) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("capture: portal handshake failed: {err}");
                return;
            }
        };
        if let Err(err) = run_pipewire(node_id, fd, shared_frame, proxy, cancel) {
            eprintln!("capture: pipewire stream failed: {err}");
        }

        // Explicitly close the portal session now that we're done with it -
        // dropping `session` only tears down our local handle, not the
        // portal-side session, and a session the portal still thinks is
        // active blocks a subsequent one from being created (this is what
        // made the monitor-switch hotkey hang with no picker on the 2nd try).
        rt.block_on(async {
            let _ = session.close().await;
        });
        eprintln!("capture: [portal] session closed");
    })
}

fn read_restore_token() -> Option<String> {
    let path = restore_token_path()?;
    fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn save_restore_token(token: &str) {
    let Some(path) = restore_token_path() else {
        eprintln!("capture: no state directory available, can't save restore token");
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("capture: failed to create state directory: {err}");
            return;
        }
    }
    if let Err(err) = fs::write(path, token) {
        eprintln!("capture: failed to save restore token: {err}");
    }
}

/// Runs the ScreenCast portal handshake and returns the PipeWire node id
/// and fd for the selected monitor's stream, plus the portal `Session` -
/// keep it alive for as long as the stream is used, then `.close()` it.
async fn open_portal() -> ashpd::Result<(u32, OwnedFd, ashpd::desktop::Session<Screencast>)> {
    eprintln!("capture: [portal] connecting to Screencast portal");
    let proxy = Screencast::new().await?;
    eprintln!("capture: [portal] creating session");
    let session = proxy.create_session(Default::default()).await?;

    let restore_token = read_restore_token();
    eprintln!(
        "capture: [portal] select_sources (restore_token present: {})",
        restore_token.is_some()
    );
    proxy
        .select_sources(
            &session,
            SelectSourcesOptions::default()
                .set_cursor_mode(CursorMode::Embedded)
                .set_sources(BitFlags::from(SourceType::Monitor))
                .set_multiple(false)
                .set_restore_token(restore_token.as_deref())
                .set_persist_mode(PersistMode::ExplicitlyRevoked),
        )
        .await?;
    eprintln!("capture: [portal] select_sources done, starting (picker may show now)");

    let response = proxy
        .start(&session, None, Default::default())
        .await?
        .response()?;
    eprintln!("capture: [portal] start done, got response");

    if let Some(token) = response.restore_token() {
        save_restore_token(token);
    }

    let stream = response
        .streams()
        .first()
        .expect("portal returned no streams (was the picker cancelled?)")
        .to_owned();

    let fd = proxy.open_pipe_wire_remote(&session, Default::default()).await?;

    Ok((stream.pipe_wire_node_id(), fd, session))
}

struct StreamState {
    format: VideoInfoRaw,
    shared_frame: Arc<Mutex<Option<CapturedFrame>>>,
    proxy: EventLoopProxy<UserEvent>,
}

/// Connects to the PipeWire node and runs its main loop until `cancel` is
/// set, decoding frames as they arrive.
fn run_pipewire(
    node_id: u32,
    fd: OwnedFd,
    shared_frame: Arc<Mutex<Option<CapturedFrame>>>,
    proxy: EventLoopProxy<UserEvent>,
    cancel: Arc<AtomicBool>,
) -> Result<(), pw::Error> {
    pw::init();

    let mainloop = MainLoopBox::new(None)?;
    let context = ContextBox::new(mainloop.loop_(), None)?;
    let core = context.connect_fd(fd, None)?;

    let state = StreamState {
        format: Default::default(),
        shared_frame,
        proxy,
    };

    let stream = StreamBox::new(
        &core,
        "pipsqueak-monitor-capture",
        properties! {
            *pw::keys::MEDIA_TYPE => "Video",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Screen",
        },
    )?;

    let _listener = stream
        .add_local_listener_with_user_data(state)
        .param_changed(|_, state, id, param| {
            let Some(param) = param else {
                return;
            };
            if id != spa::param::ParamType::Format.as_raw() {
                return;
            }

            let (media_type, media_subtype) = match format_utils::parse_format(param) {
                Ok(v) => v,
                Err(_) => return,
            };
            if media_type != spa::param::format::MediaType::Video
                || media_subtype != spa::param::format::MediaSubtype::Raw
            {
                return;
            }

            if let Err(err) = state.format.parse(param) {
                eprintln!("capture: failed to parse negotiated video format: {err}");
                return;
            }
            println!(
                "capture: negotiated format {:?} ({}x{})",
                state.format.format(),
                state.format.size().width,
                state.format.size().height,
            );
        })
        .process(|stream, state| {
            let Some(mut buffer) = stream.dequeue_buffer() else {
                return;
            };
            let Some(data) = buffer.datas_mut().first_mut() else {
                return;
            };
            if let Some(frame) = decode_frame(&state.format, data) {
                *state.shared_frame.lock().unwrap() = Some(frame);
                let _ = state.proxy.send_event(UserEvent::FrameReady);
            }
        })
        .register()?;

    // Offer the formats/sizes/framerates we're prepared to decode; the
    // compositor picks whichever it can actually deliver.
    let obj = spa::pod::object!(
        SpaTypes::ObjectParamFormat,
        spa::param::ParamType::EnumFormat,
        spa::pod::property!(
            spa::param::format::FormatProperties::MediaType,
            Id,
            spa::param::format::MediaType::Video
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::MediaSubtype,
            Id,
            spa::param::format::MediaSubtype::Raw
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            VideoFormat::BGRx,
            VideoFormat::BGRx,
            VideoFormat::RGBx,
            VideoFormat::RGBA,
            VideoFormat::BGRA,
            VideoFormat::RGB,
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            Rectangle { width: 1920, height: 1080 },
            Rectangle { width: 1, height: 1 },
            Rectangle { width: 8192, height: 8192 }
        ),
        spa::pod::property!(
            spa::param::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            Fraction { num: 30, denom: 1 },
            Fraction { num: 0, denom: 1 },
            Fraction { num: 1000, denom: 1 }
        ),
    );
    let values: Vec<u8> = spa::pod::serialize::PodSerializer::serialize(
        Cursor::new(Vec::new()),
        &spa::pod::Value::Object(obj),
    )
    .expect("failed to serialize format pod")
    .0
    .into_inner();
    let mut params = [spa::pod::Pod::from_bytes(&values).expect("invalid format pod")];

    stream.connect(
        Direction::Input,
        Some(node_id),
        pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
        &mut params,
    )?;

    eprintln!("capture: [pipewire] entering poll loop");
    while !cancel.load(Ordering::Relaxed) {
        mainloop.loop_().iterate(Timeout::Finite(Duration::from_millis(200)));
    }
    eprintln!("capture: [pipewire] cancelled, exiting poll loop");
    Ok(())
}

/// Decodes one PipeWire buffer plane into a `CapturedFrame`, converting
/// whichever raw RGB-ish format was negotiated into the packed u32 layout
/// `softbuffer` wants. Returns `None` for formats we don't handle (e.g. the
/// YUV ones) or an incomplete/malformed buffer, in which case the caller
/// just keeps showing the previous frame.
fn decode_frame(format: &VideoInfoRaw, data: &mut spa::buffer::Data) -> Option<CapturedFrame> {
    let width = format.size().width;
    let height = format.size().height;
    if width == 0 || height == 0 {
        return None;
    }

    let bytes_per_pixel: usize = match format.format() {
        VideoFormat::BGRx | VideoFormat::RGBx | VideoFormat::RGBA | VideoFormat::BGRA => 4,
        VideoFormat::RGB => 3,
        _ => return None,
    };
    let swap_rb = matches!(format.format(), VideoFormat::BGRx | VideoFormat::BGRA);

    let row_bytes = width as usize * bytes_per_pixel;
    let stride = data.chunk().stride();
    let stride = if stride > 0 { stride as usize } else { row_bytes };
    if stride < row_bytes {
        return None;
    }

    let bytes = data.data()?;
    if bytes.len() < stride * (height as usize).saturating_sub(1) + row_bytes {
        return None; // short/partial buffer
    }

    let mut pixels = vec![0u32; (width * height) as usize];
    for y in 0..height as usize {
        let row = &bytes[y * stride..y * stride + row_bytes];
        for x in 0..width as usize {
            let px = &row[x * bytes_per_pixel..x * bytes_per_pixel + bytes_per_pixel];
            let (r, g, b) = if swap_rb {
                (px[2], px[1], px[0])
            } else {
                (px[0], px[1], px[2])
            };
            pixels[y * width as usize + x] =
                0xFF00_0000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
        }
    }

    Some(CapturedFrame { width, height, pixels })
}
