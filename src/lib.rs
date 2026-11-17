//! The amenui website: the landing page (the Body scene), the Lab, the showcase scenes, a page for every
//! component, material, mark set and preset, and the documentation, all drawn by amenui itself. The same
//! code runs in a native window (`window`) and on the web (`web/`, built to one canvas).
pub mod demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod drive;
pub mod gallery;
pub mod showcase;

/// The site's message catalogs (`assets/i18n/`), installed over amenui's own.
pub const CATALOGS: [(amenui::i18n::Locale, &str, &str); 2] = [
    (amenui::i18n::Locale::ZhHans, "assets/i18n/zh-Hans.txt", include_str!("../assets/i18n/zh-Hans.txt")),
    (amenui::i18n::Locale::En, "assets/i18n/en.txt", include_str!("../assets/i18n/en.txt")),
];
/// Install the site's catalogs, once (`gallery::new_ui` and the Body scene call it).
pub fn install_catalogs() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| amenui::i18n::install(&CATALOGS).unwrap_or_else(|e| panic!("{e}")));
}

/// Web entry: the page is a canvas that forwards pointer, wheel, key and resize events here, and the
/// page's `location.hash` (the gallery page, so it can be linked). Positions are CSS (logical) px;
/// sizes are device px.
#[cfg(target_arch = "wasm32")]
mod web {
    use crate::demo;
    use crate::gallery::{self, GalleryUi};
    use amenui::el::Key;
    use amenui::render::Renderer;
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;

    struct State { ui: GalleryUi, r: Renderer, surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration, started: bool }
    thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; }

    #[wasm_bindgen]
    pub async fn start(canvas: web_sys::HtmlCanvasElement, sans: Vec<u8>, pixel: Vec<u8>, mono: Vec<u8>, scale: f32, hash: String) -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        let (w, h) = (canvas.width(), canvas.height());
        // WebGPU where an adapter really exists, WebGL2 otherwise.
        let instance = wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor::new_without_display_handle()).await;
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas)).map_err(|e| e.to_string())?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() }).await.map_err(|e| e.to_string())?;
        // WebGPU gets compute (the particle simulation's tier); WebGL2 has none.
        let base = if adapter.get_info().backend == wgpu::Backend::BrowserWebGpu { wgpu::Limits::downlevel_defaults() } else { wgpu::Limits::downlevel_webgl2_defaults() };
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: base.using_resolution(adapter.limits()), ..Default::default()
        }).await.map_err(|e| e.to_string())?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| !f.is_srgb()).unwrap_or(caps.formats[0]);
        let config = surface.get_default_config(&adapter, w, h).map(|mut c| { c.format = format; c }).ok_or("surface not supported")?;
        surface.configure(&device, &config);
        let mut r = Renderer::new(device, queue, format, w, h);
        r.set_particle_tier(amenui::render::particle_tier(&adapter));
        for (id, pw, ph, px) in demo::pictures() { r.set_picture(id, pw, ph, &px); }
        let mut ui = gallery::new_ui(amenui::text::with_serif(amenui::text::with_geist(amenui::text::bundled(sans, pixel, mono), amenui::text::geist_bytes()), amenui::text::serif_bytes()), w, h, scale);
        // The Body scene is the landing page; any other page is one hash away (`#gallery` is the gallery's first, `#lab` the Lab).
        let hash = if hash.trim_start_matches('#').is_empty() { "body".to_owned() } else { hash };
        if ui.state.route(&hash) { ui.relayout(); }
        ui.perf_mut().backend = backend_name(adapter.get_info().backend).into();
        ui.perf_mut().adapter = adapter.get_info().name;
        STATE.with(|s| *s.borrow_mut() = Some(State { ui, r, surface, config, started: false }));
        Ok(())
    }
    fn with<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> { STATE.with(|s| s.borrow_mut().as_mut().map(f)) }

    #[wasm_bindgen] pub fn resize(w: u32, h: u32, scale: f32) {
        with(|s| { s.config.width = w.max(1); s.config.height = h.max(1); s.surface.configure(s.r.device(), &s.config); s.ui.resize(w, h, scale); });
    }
    // Every input first brings the clock to its time: frames are drawn on demand, so the last tick may
    // be long ago, and what the input starts (a burst, a front) starts now.
    // Pointers of every kind: `kind` is 0 mouse, 1 touch, 2 pen (DESIGN.md §1.8.2).
    fn kind(k: u32) -> amenui::ui::PointerKind { match k { 1 => amenui::ui::PointerKind::Touch, 2 => amenui::ui::PointerKind::Pen, _ => amenui::ui::PointerKind::Mouse } }
    /// `te` is when the event happened (its timestamp), `t` when it is handled.
    #[wasm_bindgen] pub fn pointer_move(id: u32, k: u32, x: f32, y: f32, t: f32, te: f32) { with(|s| { s.ui.tick(t); s.ui.event_time(te.min(t)); s.ui.pointer_moved(id as u64, kind(k), x, y) }); }
    #[wasm_bindgen] pub fn pointer_leave(t: f32) { with(|s| { s.ui.tick(t); s.ui.pointer_leave() }); }
    #[wasm_bindgen] pub fn pointer_down(id: u32, k: u32, x: f32, y: f32, shift: bool, t: f32, te: f32) { with(|s| { s.ui.tick(t); s.ui.event_time(te.min(t)); s.ui.pointer_down(id as u64, kind(k), x, y, shift); }); }
    #[wasm_bindgen] pub fn pointer_up(id: u32, k: u32, x: f32, y: f32, t: f32) { with(|s| { s.ui.tick(t); s.ui.pointer_up(id as u64, kind(k), x, y) }); }
    #[wasm_bindgen] pub fn pointer_cancel(id: u32, t: f32) { with(|s| { s.ui.tick(t); s.ui.pointer_cancel(id as u64) }); }
    // The accessibility mirror (DESIGN.md §1.14): the page keeps a hidden DOM tree of these.
    /// What changed in the mirror since the last call, as JSON: `[{"set": {...}}, {"remove": key}, {"focus": key}]`.
    #[wasm_bindgen] pub fn a11y_update() -> String { with(|s| a11y_json(&s.ui.a11y_diff())).unwrap_or_else(|| "[]".into()) }
    /// Assistive technology focused or activated a mirrored element.
    #[wasm_bindgen] pub fn a11y_focus(key: &str, t: f32) { with(|s| { s.ui.tick(t); s.ui.focus_key(key) }); }
    #[wasm_bindgen] pub fn a11y_activate(key: &str, t: f32) -> bool { with(|s| { s.ui.tick(t); s.ui.activate(key) }).unwrap_or(false) }
    fn a11y_json(ops: &[amenui::ui::A11yOp]) -> String {
        use amenui::ui::A11yOp;
        let q = |s: &str| { let mut o = String::from("\""); for c in s.chars() { match c { '"' => o.push_str("\\\""), '\\' => o.push_str("\\\\"), c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)), c => o.push(c) } } o.push('"'); o };
        let parts: Vec<String> = ops.iter().map(|op| match op {
            A11yOp::Remove(k) => format!("{{\"remove\":{}}}", q(k)),
            A11yOp::Focus(k) => format!("{{\"focus\":{}}}", k.as_deref().map_or("null".into(), q)),
            A11yOp::Set(n) => {
                let mut f = vec![format!("\"key\":{}", q(&n.key)), format!("\"role\":\"{}\"", n.role.aria()), format!("\"name\":{}", q(&n.name)),
                    format!("\"rect\":[{},{},{},{}]", n.rect.x, n.rect.y, n.rect.w, n.rect.h)];
                if let Some(p) = &n.parent { f.push(format!("\"parent\":{}", q(p))); }
                if let Some((v, lo, hi)) = n.value { f.push(format!("\"value\":[{v},{lo},{hi}]")); }
                if let Some(t) = &n.text { f.push(format!("\"text\":{}", q(t))); }
                if let Some(c) = n.checked { f.push(format!("\"checked\":{c}")); }
                if let Some(c) = n.selected { f.push(format!("\"selected\":{c}")); }
                if n.level > 0 { f.push(format!("\"level\":{}", n.level)); }
                if n.modal { f.push("\"modal\":true".into()); }
                if n.disabled { f.push("\"disabled\":true".into()); }
                if n.focusable { f.push("\"focusable\":true".into()); }
                format!("{{\"set\":{{{}}}}}", f.join(","))
            }
        }).collect();
        format!("[{}]", parts.join(","))
    }
    /// Whether a fling is going on, for automation and tests.
    #[wasm_bindgen] pub fn flinging() -> bool { with(|s| s.ui.flinging()).unwrap_or(false) }
    /// A scroll node's offset (CSS px), for automation and tests.
    #[wasm_bindgen] pub fn scroll_offset(key: &str) -> f32 { with(|s| s.ui.scroll_offset(key)).unwrap_or(0.0) }
    #[wasm_bindgen] pub fn wheel(x: f32, y: f32, dx: f32, dy: f32, t: f32) { with(|s| { s.ui.tick(t); s.ui.wheel_xy(x, y, dx, dy) }); }
    /// The page's hash changed (a link, Back): show that page. Returns whether it names one.
    #[wasm_bindgen] pub fn route(hash: &str) -> bool {
        with(|s| { let ok = s.ui.state.route(hash); if ok { s.ui.relayout(); } ok }).unwrap_or(false)
    }
    /// Theme and material by name (as the render tool takes them; empty leaves one as it is), from a
    /// host such as `<amenui-view theme="dark" material="dot">`.
    #[wasm_bindgen] pub fn set_look(theme: &str, material: &str) {
        with(|s| {
            if let Ok(t) = amenui::style::theme_named(theme) { gallery::update(&mut s.ui.state, gallery::Msg::Theme(t)); }
            if let Ok(m) = amenui::style::material_named(material) { gallery::update(&mut s.ui.state, gallery::Msg::Material(m.id() as usize)); }
            s.ui.relayout();
        });
    }
    /// The language by tag (`zh-Hans`, `en`); an unknown tag changes nothing.
    #[wasm_bindgen] pub fn set_locale(tag: &str) {
        with(|s| { if let Some(l) = amenui::i18n::Locale::from_tag(tag) { gallery::update(&mut s.ui.state, gallery::Msg::Locale(l)); s.ui.relayout(); } });
    }
    /// The current page's hash name, for the page to put in `location.hash`.
    /// The readout's choice: 0 hidden, 1 shown, 2 with more (the page keeps it in localStorage).
    /// The frames of the last few seconds: how many, and their mean and largest cost (ms: update and
    /// layout, scene, submit together), and the mean time between them (ms) — for measuring a page headless.
    #[wasm_bindgen] pub fn frame_costs() -> Vec<f32> {
        with(|s| { let f = &s.ui.perf().frames; let v: Vec<f32> = f.iter().map(|(_, c)| c.total()).collect(); let gap = if f.len() > 1 { ((f[f.len() - 1].0 - f[0].0) / (f.len() - 1) as f64) as f32 } else { 0.0 }; vec![v.len() as f32, v.iter().sum::<f32>() / v.len().max(1) as f32, v.iter().copied().fold(0.0, f32::max), gap] }).unwrap_or_default()
    }
    /// The viewer's reduced-motion preference: the Lab's pieces then rest after a moment (DESIGN.md §5.3).
    #[wasm_bindgen] pub fn set_reduced_motion(on: bool) { with(|s| { s.ui.state.lab.reduced = on; s.ui.state.body.reduced = on; s.ui.set_reduced_motion(on); }); }
    #[wasm_bindgen] pub fn readout() -> u8 { with(|s| s.ui.state.overlay).unwrap_or(0) }
    #[wasm_bindgen] pub fn set_readout(v: u8) { with(|s| { s.ui.state.overlay = v.min(2); s.ui.relayout(); }); }
    #[wasm_bindgen] pub fn hash() -> String { with(|s| s.ui.state.page.hash()).unwrap_or_default() }
    /// A DOM key name and Shift. Returns whether the page should not act on it itself.
    #[wasm_bindgen] pub fn key(name: &str, shift: bool, t: f32) -> bool {
        let Some(k) = Key::from_name(name) else { return false };
        with(|s| { s.ui.tick(t); s.ui.key(k, shift) }).unwrap_or(false)
    }
    /// A DOM key name with its modifiers (`cmd` is Ctrl or ⌘). Returns whether the page should not act
    /// on it itself.
    #[wasm_bindgen] pub fn key_mods(name: &str, shift: bool, cmd: bool, alt: bool, t: f32) -> bool {
        let Some(k) = Key::from_name(name) else { return false };
        with(|s| { s.ui.tick(t); s.ui.key_mods(k, amenui::el::Mods { shift, cmd, alt }) }).unwrap_or(false)
    }
    // Text (DESIGN.md §1.8.1): the page's hidden text element forwards typed text, the input method's
    // composition and the clipboard here while a text field has focus.
    #[wasm_bindgen] pub fn text_focused() -> bool { with(|s| s.ui.text_focused()).unwrap_or(false) }
    #[wasm_bindgen] pub fn text_input(text: &str, t: f32) -> bool { with(|s| { s.ui.tick(t); s.ui.text_input(text) }).unwrap_or(false) }
    #[wasm_bindgen] pub fn ime_preedit(text: &str, t: f32) -> bool { with(|s| { s.ui.tick(t); s.ui.ime_preedit(text, None) }).unwrap_or(false) }
    #[wasm_bindgen] pub fn ime_commit(text: &str, t: f32) -> bool { with(|s| { s.ui.tick(t); s.ui.ime_commit(text) }).unwrap_or(false) }
    #[wasm_bindgen] pub fn paste(text: &str, t: f32) -> bool { with(|s| { s.ui.tick(t); s.ui.paste(text) }).unwrap_or(false) }
    #[wasm_bindgen] pub fn copy() -> Option<String> { with(|s| s.ui.copy()).flatten() }
    /// Text a copy button asked to put on the clipboard (once).
    #[wasm_bindgen] pub fn take_clipboard() -> Option<String> { with(|s| s.ui.state.clipboard.take()).flatten() }
    #[wasm_bindgen] pub fn cut(t: f32) -> Option<String> { with(|s| { s.ui.tick(t); s.ui.cut() }).flatten() }
    /// Where the focused field's caret is (x, y, w, h, CSS px), or nothing when focus is not on one.
    #[wasm_bindgen] pub fn ime_rect() -> Vec<f32> { with(|s| s.ui.ime_rect().map(|r| vec![r.x, r.y, r.w, r.h]).unwrap_or_default()).unwrap_or_default() }
    /// Where the element with this key is (x, y, w, h, CSS px), or nothing: for automation and tests.
    #[wasm_bindgen] pub fn node_rect(key: &str) -> Vec<f32> { with(|s| s.ui.node(key).map(|n| vec![n.rect.x, n.rect.y, n.rect.w, n.rect.h]).unwrap_or_default()).unwrap_or_default() }
    /// The CSS cursor for where the pointer is.
    #[wasm_bindgen] pub fn cursor() -> String { with(|s| s.ui.cursor().css().to_owned()).unwrap_or_else(|| "default".into()) }
    /// A DOM key name let go.
    #[wasm_bindgen] pub fn key_up(name: &str, t: f32) {
        let Some(k) = Key::from_name(name) else { return };
        with(|s| { s.ui.tick(t); s.ui.key_up(k) });
    }
    /// Frames drawn and wakes that found nothing to draw, since the start.
    #[wasm_bindgen] pub fn frames() -> Vec<u32> { with(|s| vec![s.ui.perf().drawn as u32, s.ui.perf().skipped as u32]).unwrap_or_default() }
    fn backend_name(b: wgpu::Backend) -> &'static str { match b { wgpu::Backend::BrowserWebGpu => "WebGPU", wgpu::Backend::Gl => "WebGL2", _ => "other" } }
    /// Draw a frame if one is needed. Returns when the next one is: 0 as soon as possible (keep the
    /// animation frame loop going), a delay in seconds (a timed wake), or -1 (only input can change
    /// anything: stop until then).
    #[wasm_bindgen] pub fn frame(t: f32) -> f32 {
        with(|s| {
            if !s.started { s.ui.restart(t); s.started = true; }
            s.ui.tick(t);
            if s.ui.needs_frame(t) { draw(s); } else { s.ui.perf_mut().skipped += 1; }
            s.ui.next_wake(t).map_or(-1.0, |w| (w - t).max(0.0))
        }).unwrap_or(-1.0)
    }
    fn draw(s: &mut State) {
        {
            let tex = match s.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                _ => { s.surface.configure(s.r.device(), &s.config); return; }
            };
            let view = tex.texture.create_view(&Default::default());
            let look = s.ui.state.look();
            s.r.draw(&view, &mut s.ui, &look);
            drop(view);
            s.r.queue().present(tex);
        }
    }
}
