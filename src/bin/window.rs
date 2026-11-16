//! The gallery in a native window (winit + wgpu): Metal on macOS, Vulkan elsewhere. It forwards the
//! pointer, wheel, keys and size to the framework, as the web page does.
//! usage: cargo run --release --bin window [page] [--shot <path.png>@<ms> …] [--exit-after <ms>]
//!   page: a hash name, e.g. dialog. `--shot` reads the window's own frame back at that many ms after
//!   start and saves it (any number of them); `--exit-after` closes the window after that many ms.
//!   It reports the adapter and, every two seconds, how many frames it drew, on stderr.
use lab::el::Key;
use lab::gallery::{self, GalleryUi};
use lab::render::Renderer;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use lab::ui::PointerKind;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{Key as WKey, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

struct Running {
    window: Arc<Window>,
    frames: u64,
    /// Redraw requests that found nothing to draw.
    skipped: u64,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    r: Renderer,
    ui: GalleryUi,
}

struct App {
    display: OwnedDisplayHandle,
    page: Option<String>,
    run: Option<Running>,
    start: Instant,
    mods: ModifiersState,
    /// An input method is composing (its keys are its own).
    composing: bool,
    /// Logical px.
    cursor: (f32, f32),
    /// Frames to save (path, ms after start), soonest first; when to close.
    shots: Vec<(String, f32)>,
    exit_after: Option<f32>,
    reported: f32,
}

impl App {
    fn now(&self) -> f32 { self.start.elapsed().as_secs_f32() }
    /// Every two seconds: how many frames were drawn and how many redraws found nothing to draw.
    fn report(&mut self, now: f32) {
        if now - self.reported < 2.0 { return; }
        self.reported = now;
        if let Some(s) = &self.run { eprintln!("{:6.1} s: {} frames, {} skipped", now, s.frames, s.skipped); }
    }

    fn open(&mut self, el: &ActiveEventLoop) -> Result<Running, String> {
        let attrs = Window::default_attributes().with_title("gui-lab").with_inner_size(LogicalSize::new(1280.0, 860.0));
        let window = Arc::new(el.create_window(attrs).map_err(|e| e.to_string())?);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(Box::new(self.display.clone())));
        let surface = instance.create_surface(window.clone()).map_err(|e| e.to_string())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() })).map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        eprintln!("adapter: {} ({:?}, {:?})", info.name, info.backend, info.device_type);
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).map_err(|e| e.to_string())?;
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        // A linear format: the material writes the colours it means (it encodes for sRGB surfaces itself).
        let format = caps.formats.iter().copied().find(|f| !f.is_srgb()).unwrap_or(caps.formats[0]);
        let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).ok_or("surface not supported")?;
        config.format = format;
        surface.configure(&device, &config);
        eprintln!("surface: {}×{} {:?}, present mode {:?}", config.width, config.height, config.format, config.present_mode);
        let mut r = Renderer::new(device, queue, format, config.width, config.height);
        r.set_particle_tier(lab::render::particle_tier(&adapter));
        for (id, pw, ph, px) in lab::demo::pictures() { r.set_picture(id, pw, ph, &px); }
        let faces = lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).map_err(|e| e.to_string())?;
        let mut ui = gallery::new_ui(faces, config.width, config.height, window.scale_factor() as f32);
        ui.perf_mut().backend = format!("{:?}", info.backend);
        ui.perf_mut().adapter = info.name.clone();
        if let Some(p) = &self.page && ui.state.route(p) { ui.relayout(); }
        ui.restart(self.now());
        Ok(Running { window, frames: 0, skipped: 0, surface, config, r, ui })
    }
}

/// A winit key as the framework's key.
fn key(k: &WKey) -> Vec<Key> {
    match k {
        WKey::Named(n) => match n {
            NamedKey::Enter => vec![Key::Enter], NamedKey::Backspace => vec![Key::Backspace], NamedKey::Delete => vec![Key::Delete],
            NamedKey::Escape => vec![Key::Escape], NamedKey::Tab => vec![Key::Tab], NamedKey::Space => vec![Key::Char(' ')],
            NamedKey::ArrowUp => vec![Key::Up], NamedKey::ArrowDown => vec![Key::Down], NamedKey::ArrowLeft => vec![Key::Left], NamedKey::ArrowRight => vec![Key::Right],
            NamedKey::F2 => vec![Key::F(2)], NamedKey::Home => vec![Key::Home], NamedKey::End => vec![Key::End], NamedKey::PageUp => vec![Key::PageUp], NamedKey::PageDown => vec![Key::PageDown],
            _ => vec![],
        },
        WKey::Character(s) => s.chars().map(Key::Char).collect(),
        _ => vec![],
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.run.is_some() { return; }
        match self.open(el) {
            Ok(r) => self.run = Some(r),
            Err(e) => { eprintln!("could not open a window: {e}"); el.exit(); }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let now = self.now();
        let Some(s) = self.run.as_mut() else { return };
        let scale = s.window.scale_factor() as f32;
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(size) => {
                s.config.width = size.width.max(1);
                s.config.height = size.height.max(1);
                s.surface.configure(s.r.device(), &s.config);
                s.ui.resize(s.config.width, s.config.height, scale);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => s.ui.resize(s.config.width, s.config.height, scale_factor as f32),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as f32 / scale, position.y as f32 / scale);
                s.ui.tick(now);
                s.ui.pointer_move(self.cursor.0, self.cursor.1);
            }
            WindowEvent::CursorLeft { .. } => { s.ui.tick(now); s.ui.pointer_leave() }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let (x, y) = self.cursor;
                s.ui.tick(now);
                if state == ElementState::Pressed { s.ui.press_with(x, y, self.mods.shift_key()) } else { s.ui.release(x, y) }
            }
            // Touch screens: fingers scroll and tap (DESIGN.md §1.8.2). Pinch and other gestures are not
            // handled (their events fall through below).
            WindowEvent::Touch(t) => {
                let (x, y) = (t.location.x as f32 / scale, t.location.y as f32 / scale);
                s.ui.tick(now);
                match t.phase {
                    TouchPhase::Started => s.ui.pointer_down(t.id, PointerKind::Touch, x, y, false),
                    TouchPhase::Moved => s.ui.pointer_moved(t.id, PointerKind::Touch, x, y),
                    TouchPhase::Ended => s.ui.pointer_up(t.id, PointerKind::Touch, x, y),
                    TouchPhase::Cancelled => s.ui.pointer_cancel(t.id),
                }
            }
            // The input method: its composition is shown in the field, its commit inserted.
            WindowEvent::Ime(ime) => {
                s.ui.tick(now);
                match ime {
                    Ime::Preedit(text, cursor) => { self.composing = !text.is_empty(); s.ui.ime_preedit(&text, cursor); }
                    Ime::Commit(text) => { self.composing = false; s.ui.ime_commit(&text); }
                    Ime::Disabled => if self.composing { self.composing = false; s.ui.ime_preedit("", None); },
                    Ime::Enabled => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // winit: positive is away from the user (up / left); the framework scrolls down for positive.
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (-x * 40.0, -y * 40.0),
                    MouseScrollDelta::PixelDelta(p) => (-p.x as f32 / scale, -p.y as f32 / scale),
                };
                s.ui.wheel_xy(self.cursor.0, self.cursor.1, dx, dy);
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                s.ui.tick(now);
                let cmd = if cfg!(target_os = "macos") { self.mods.super_key() } else { self.mods.control_key() };
                let mods = lab::el::Mods { shift: self.mods.shift_key(), cmd, alt: self.mods.alt_key() };
                if cmd {
                    // The clipboard, through the system's (arboard); Cmd+A / Z / Y go to the text field.
                    match &event.logical_key {
                        WKey::Character(c) if c.eq_ignore_ascii_case("c") => { if let Some(t) = s.ui.copy() { clipboard(|b| b.set_text(t).is_ok()); } }
                        WKey::Character(c) if c.eq_ignore_ascii_case("x") => { if let Some(t) = s.ui.cut() { clipboard(|b| b.set_text(t).is_ok()); } }
                        WKey::Character(c) if c.eq_ignore_ascii_case("v") => { if let Some(t) = clipboard(|b| b.get_text().ok()).flatten() { s.ui.paste(&t); } }
                        k => for k in key(k) { s.ui.key_mods(k, mods); },
                    }
                    return;
                }
                // While the input method composes, the keys are its own.
                if self.composing { return; }
                // In a text field, characters are text (what the key produced, with the layout's shifts).
                if s.ui.text_focused() && !mods.alt && let Some(t) = event.text.as_ref().filter(|t| t.chars().all(|c| !c.is_control())) {
                    s.ui.text_input(t);
                    return;
                }
                if mods.alt { return; }
                for k in key(&event.logical_key) { s.ui.key_mods(k, mods); }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                s.ui.tick(now);
                for k in key(&event.logical_key) { s.ui.key_up(k); }
            }
            WindowEvent::RedrawRequested => {
                s.ui.tick(now);
                self.report(now);
                let Some(s) = self.run.as_mut() else { return };
                if self.exit_after.is_some_and(|t| now * 1000.0 >= t) {
                    eprintln!("closing after {} frames", s.frames);
                    el.exit();
                    return;
                }
                // A frame to save: drawn into a texture of the surface's format and read back.
                if self.shots.first().is_some_and(|(_, t)| now * 1000.0 >= *t) {
                    let (path, _) = self.shots.remove(0);
                    let look = s.ui.state.look();
                    match read_back(&mut s.r, &mut s.ui, &look, s.config.format, s.config.width, s.config.height) {
                        Ok(px) => { lab::offscreen::save_png(&path, s.config.width, s.config.height, &px); eprintln!("saved {path} at {:.1} s", now); }
                        Err(e) => eprintln!("could not read the frame back: {e}"),
                    }
                    // The saved frame was not presented: the window still has to show it.
                    s.ui.invalidate();
                    s.window.request_redraw();
                    return;
                }
                // Nothing changed and nothing moves: no frame (the event loop waits for input or a wake).
                if !s.ui.needs_frame(now) { s.skipped += 1; s.ui.perf_mut().skipped += 1; return; }
                // The input method follows the focused field's caret; the pointer's shape follows what is under it.
                match s.ui.ime_rect() {
                    Some(r) => {
                        s.window.set_ime_allowed(true);
                        s.window.set_ime_cursor_area(winit::dpi::LogicalPosition::new(r.x, r.y), winit::dpi::LogicalSize::new(r.w.max(1.0), r.h));
                    }
                    None => s.window.set_ime_allowed(false),
                }
                s.window.set_cursor(cursor_icon(s.ui.cursor()));
                let t0 = Instant::now();
                let frame = match s.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    other => {
                        eprintln!("no surface texture ({:?}) after {:.0} ms; configuring again", std::mem::discriminant(&other), t0.elapsed().as_secs_f32() * 1000.0);
                        s.surface.configure(s.r.device(), &s.config);
                        return;
                    }
                };
                if std::env::var_os("WINDOW_TRACE").is_some() { eprintln!("acquire {:.0} ms", t0.elapsed().as_secs_f32() * 1000.0); }
                let view = frame.texture.create_view(&Default::default());
                let look = s.ui.state.look();
                s.r.draw(&view, &mut s.ui, &look);
                drop(view);
                s.r.queue().present(frame);
                s.frames += 1;
                // A copy button asked for the clipboard.
                if let Some(t) = s.ui.state.clipboard.take() { let _ = clipboard(|c| c.set_text(t)); }
                if std::env::var_os("WINDOW_TRACE").is_some() { eprintln!("frame {:.0} ms", t0.elapsed().as_secs_f32() * 1000.0); }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        // Frames on demand: draw when the runtime needs one, else sleep until its next timed wake (the
        // caret's blink, a clock), the next report, a frame to save or the time to close.
        let now = self.now();
        self.report(now);
        let Some(s) = &self.run else { return };
        let mut wake = s.ui.next_wake(now);
        if wake.is_some_and(|w| w <= now) { s.window.request_redraw(); wake = None; }
        let others = [Some(self.reported + 2.0), self.shots.first().map(|(_, t)| t / 1000.0), self.exit_after.map(|t| t / 1000.0)];
        let next = others.into_iter().flatten().chain(wake).fold(f32::INFINITY, f32::min);
        let shot_due = self.shots.first().is_some_and(|(_, t)| now * 1000.0 >= *t) || self.exit_after.is_some_and(|t| now * 1000.0 >= t);
        if shot_due { s.window.request_redraw(); }
        el.set_control_flow(if next.is_finite() { ControlFlow::WaitUntil(self.start + std::time::Duration::from_secs_f32(next.max(now))) } else { ControlFlow::Wait });
    }
}

/// Use the system clipboard (opened for each use; `None` when there is none).
fn clipboard<R>(f: impl FnOnce(&mut arboard::Clipboard) -> R) -> Option<R> {
    match arboard::Clipboard::new() {
        Ok(mut b) => Some(f(&mut b)),
        Err(e) => { eprintln!("no clipboard: {e}"); None }
    }
}

fn cursor_icon(c: lab::el::Cursor) -> winit::window::CursorIcon {
    use lab::el::Cursor as C;
    use winit::window::CursorIcon as I;
    match c { C::Default => I::Default, C::Pointer => I::Pointer, C::Text => I::Text, C::Grab => I::Grab, C::Grabbing => I::Grabbing, C::ColResize => I::ColResize, C::RowResize => I::RowResize }
}

/// Draw a frame into an off-screen texture of the surface's format and read it back as RGBA8.
fn read_back(r: &mut Renderer, ui: &mut GalleryUi, look: &lab::style::Look, format: wgpu::TextureFormat, w: u32, h: u32) -> Result<Vec<u8>, String> {
    let device = r.device().clone();
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let view = tex.create_view(&Default::default());
    r.draw(&view, ui, look);
    let row = (w * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
    r.queue().submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| e.to_string())?;
    let data = buf.slice(..).get_mapped_range().map_err(|e| format!("{e:?}"))?;
    let bgra = matches!(format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb);
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for p in data[(y * row) as usize..(y * row + w * 4) as usize].chunks(4) {
            if bgra { px.extend_from_slice(&[p[2], p[1], p[0], 255]); } else { px.extend_from_slice(&[p[0], p[1], p[2], 255]); }
        }
    }
    Ok(px)
}

fn main() -> std::process::ExitCode {
    let el = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => { eprintln!("no window system here: {e}"); return std::process::ExitCode::FAILURE; }
    };
    let (mut page, mut shots, mut exit_after) = (None, vec![], None);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--shot" => if let Some((p, t)) = args.next().as_deref().and_then(|v| v.rsplit_once('@')) && let Ok(t) = t.parse::<f32>() { shots.push((p.to_owned(), t)); },
            "--exit-after" => exit_after = args.next().and_then(|v| v.parse().ok()),
            _ => page = Some(a),
        }
    }
    shots.sort_by(|a, b| a.1.total_cmp(&b.1));
    // The Body scene is where the window opens unless a page is named.
    let page = page.or_else(|| Some("body".to_owned()));
    let mut app = App { display: el.owned_display_handle(), page, run: None, start: Instant::now(), mods: ModifiersState::empty(), composing: false, cursor: (0.0, 0.0), shots, exit_after, reported: 0.0 };
    match el.run_app(&mut app) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => { eprintln!("{e}"); std::process::ExitCode::FAILURE }
    }
}
