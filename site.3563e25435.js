// The glue between the browser and the framework (after wasm-bindgen's own, which the build puts in
// front of this file). `mount(stage, opts)` starts the framework in a stage element (the canvas, the
// text agent and the accessibility mirror) and returns what a host can change. The site's pages
// mount once in #stage (`site()` at the end); `<amenui-view>` (embed.js) mounts in its shadow root.
// One view per document: the module keeps one interface.
//
// opts: `load` (a function giving { module, fonts: [sans, pixel, mono], texts, glue } as promises;
// `texts`, [name, body] pairs, may be left out; `glue` is the module's wasm-bindgen glue, the global
// `wasm_bindgen` when left out), `faces` (the faces the module does not carry, name → URL: each
// is fetched once, when a page first uses it), `texts` ({ texts: base URL, textsV: version }: where the
// texts the module does not carry are, each fetched when the module first asks for it), `route` (the
// first page: a path such as `components/button/`, or a hash name such as `#button`), `keys` (where
// key events come from; the window by default), `onRoute(path, hash)` (the page changed inside it),
// `started(module)` (after the module started, before its first frame), `onFrame()` (the first frame
// is drawn), `frame()` (after every frame the loop runs), `after()` (after every handled input).
//
// The pictures (photographs drawn in marks) are made when the page first draws one (`pictures`).
async function mount(stage, opts) {
  const root = stage.getRootNode();
  const active = () => (root.activeElement !== undefined ? root.activeElement : document.activeElement);
  const canvas = stage.querySelector('.c');
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const fit = () => { const r = stage.getBoundingClientRect(); canvas.width = Math.max(1, Math.floor(r.width * dpr)); canvas.height = Math.max(1, Math.floor(r.height * dpr)); };
  fit();
  const got = opts.load();
  const t0 = performance.now();
  const w = got.glue ? await got.glue : wasm_bindgen;
  // Tools that read the module (frame-time probes) find it by its usual name.
  if (got.glue) self.wasm_bindgen = w;
  await w({ module_or_path: got.module });
  const compiled = performance.now() - t0;
  const [sans, pixel, mono] = (await got.fonts).map(b => new Uint8Array(b));
  // The texts the module does not carry: those given go in before it starts; one it asks for later is
  // fetched once and the page is drawn again when it comes. What the first page asks for comes before
  // its first frame.
  const textsAsked = new Set(), redraw = { kick: () => {} };
  const putTexts = list => { for (const [n, b] of list || []) { textsAsked.add(n); if (typeof b === 'string') w.put_text(n, b); } };
  const fetchTexts = () => {
    const want = w.wanted_texts ? w.wanted_texts().split('\n').filter(n => n && !textsAsked.has(n)) : [];
    if (!want.length || !opts.texts) return null;
    return Promise.all(want.map(n => { textsAsked.add(n); return fetch(opts.texts.texts + n + '?' + opts.texts.textsV).then(r => r.ok ? r.text() : null).then(b => { if (b !== null) { w.put_text(n, b); redraw.kick(); } }, () => {}); }));
  };
  putTexts(await got.texts);
  await w.start(canvas, sans, pixel, mono, dpr, opts.route || '/');
  const asking = fetchTexts();
  if (asking) await asking;
  // After start: the pictures the first page draws go to the renderer, which start makes.
  w.pictures();
  // The page's scheme (AA.10, web/scheme.js sets it on the root before the first paint), before the
  // first frame; on a page without one (an embedding) the canvas keeps its own.
  const theme = document.documentElement.dataset.theme ? () => document.documentElement.dataset.theme === 'light' : null;
  if (theme && w.set_scheme) w.set_scheme(theme(), NaN, NaN);
  if (opts.started) opts.started(w);
  canvas.focus({ preventScroll: true });
  const now = () => performance.now() / 1000;
  const at = e => { const r = canvas.getBoundingClientRect(); return [e.clientX - r.left, e.clientY - r.top]; };
  let shown = null;
  const sync = () => { const p = w.path(); if (p === shown) return; shown = p; if (w.pace) w.pace(); if (opts.onRoute) opts.onRoute(p, w.hash()); };
  sync();
  // Frames on demand: the animation-frame loop runs only while the framework needs frames; when it
  // does not, the loop stops, and input or a timed wake (the caret's blink, a clock) starts it again.
  let raf = 0, timer = 0, first = true;
  const loop = () => {
    raf = 0; if (timer) { clearTimeout(timer); timer = 0; }
    const t = now();
    let d = w.frame(t);
    // A picture drawn for the first time is made, and the frame drawn again before it is shown.
    if (w.pictures()) d = w.frame(t);
    fetchFaces();
    fetchTexts();
    axUpdate();
    if (first && w.frames()[0] > 0) { first = false; if (opts.onFrame) opts.onFrame(); }
    if (opts.frame) opts.frame();
    // A wait ends on an animation frame: the timer wakes a refresh early (as amenui.js does), so a frame
    // due in one refresh (30 a second on a 60 Hz display) is drawn on that refresh and not the one after.
    if (d === 0) raf = requestAnimationFrame(loop);
    else if (d > 0) timer = setTimeout(kick, Math.max(0, d * 1000 - 20));
  };
  const kick = () => { if (!raf) raf = requestAnimationFrame(loop); };
  redraw.kick = kick;
  // The renderer's warm-up goes on in idle time, whether or not the page draws (a page at rest draws
  // nothing): each idle period makes the next pieces and draws nothing; a frame only when the look's
  // material can be drawn (2). It stops once nothing is left (no 1), never on a call that traps.
  // Without requestIdleCallback, an animation frame that does warm-up work only.
  const idle = window.requestIdleCallback ? f => requestIdleCallback(f, { timeout: 200 }) : f => requestAnimationFrame(() => f(null));
  const warmIdle = d => {
    if (!w.warm_idle) return;
    const ms = d && !d.didTimeout ? Math.max(1, Math.min(d.timeRemaining(), 6)) : 6;
    // A call that traps leaves work behind: the next idle period tries again. A module busy in another
    // call answers 1 (more to do), so only a call that has nothing left ends the loop.
    let r = 1;
    try { r = w.warm_idle(ms, now()); } catch (e) { r = 1; }
    if (r & 2) kick();
    if (r & 1) idle(warmIdle);
  };
  idle(warmIdle);
  // Faces the page uses that have not come: each fetched once; when its bytes come the page takes it
  // (each line sweeps over to it).
  const asked = new Set();
  const fetchFaces = () => {
    if (!opts.faces || !w.faces_wanted) return;
    for (const name of JSON.parse(w.faces_wanted())) {
      if (asked.has(name) || !opts.faces[name]) continue;
      asked.add(name);
      fetch(opts.faces[name]).then(r => { if (!r.ok) throw new Error(r.status); return r.arrayBuffer(); })
        .then(b => { w.add_face(name, new Uint8Array(b), now()); kick(); }, () => {});
    }
  };
  // Text: while a text field has focus the hidden text element sits at its caret and has the keyboard.
  const ta = stage.querySelector('.ta');
  const syncText = () => {
    const r = w.ime_rect();
    if (r.length) {
      ta.style.left = r[0] + 'px'; ta.style.top = r[1] + 'px'; ta.style.height = Math.max(r[3], 10) + 'px'; ta.style.fontSize = Math.max(r[3] * 0.8, 10) + 'px';
      if (active() !== ta) ta.focus({ preventScroll: true });
    } else if (active() === ta) canvas.focus({ preventScroll: true });
    canvas.style.cursor = w.cursor();
  };
  // The accessibility mirror, kept in step with what the framework reports changed.
  const ax = stage.querySelector('.a11y'), els = new Map();
  const axUpdate = () => {
    const c = canvas.getBoundingClientRect();
    for (const op of JSON.parse(w.a11y_update())) {
      if (op.remove !== undefined) { const e = els.get(op.remove); if (e) { e.remove(); els.delete(op.remove); } continue; }
      if (op.focus !== undefined) {
        // Focus follows only while assistive technology is in the mirror; otherwise the canvas (or the
        // text element) keeps the keyboard.
        const e = op.focus && els.get(op.focus);
        if (e && ax.contains(active()) && active() !== e) e.focus({ preventScroll: true });
        continue;
      }
      const n = op.set;
      let e = els.get(n.key);
      if (!e) {
        e = document.createElement('div');
        els.set(n.key, e);
        e.addEventListener('click', ev => { ev.stopPropagation(); w.a11y_activate(n.key, now()); after(); });
        e.addEventListener('focus', () => { w.a11y_focus(n.key, now()); after(); });
      }
      const parent = (n.parent && els.get(n.parent)) || ax;
      if (e.parentNode !== parent) parent.appendChild(e);
      for (const a of ['role', 'aria-label', 'aria-checked', 'aria-selected', 'aria-valuenow', 'aria-valuemin', 'aria-valuemax', 'aria-modal', 'aria-level', 'aria-disabled', 'aria-expanded', 'aria-description', 'aria-live', 'aria-invalid', 'aria-required', 'tabindex']) e.removeAttribute(a);
      if (n.role === 'text') { if (e.textContent !== n.name) e.textContent = n.name; }
      else {
        e.setAttribute('role', n.role);
        e.setAttribute('aria-label', n.name);
        if (n.role === 'textbox') e.textContent = n.text || '';
      }
      if (n.checked !== undefined) e.setAttribute('aria-checked', n.checked);
      if (n.selected !== undefined) e.setAttribute('aria-selected', n.selected);
      if (n.value) { e.setAttribute('aria-valuenow', n.value[0]); e.setAttribute('aria-valuemin', n.value[1]); e.setAttribute('aria-valuemax', n.value[2]); }
      if (n.modal) e.setAttribute('aria-modal', 'true');
      if (n.level) e.setAttribute('aria-level', n.level);
      if (n.disabled) e.setAttribute('aria-disabled', 'true');
      if (n.expanded !== undefined) e.setAttribute('aria-expanded', n.expanded);
      if (n.description) e.setAttribute('aria-description', n.description);
      if (n.live) e.setAttribute('aria-live', n.live);
      if (n.invalid) e.setAttribute('aria-invalid', 'true');
      if (n.required) e.setAttribute('aria-required', 'true');
      if (n.focusable) e.setAttribute('tabindex', '-1');
      const r = n.rect;
      e.style.left = (c.left + r[0]) + 'px'; e.style.top = (c.top + r[1]) + 'px'; e.style.width = Math.max(r[2], 1) + 'px'; e.style.height = Math.max(r[3], 1) + 'px';
    }
  };
  // A copy button in the page asks for the clipboard: hand its text to the browser after the event.
  const clip = () => { const t = w.take_clipboard(); if (t != null && navigator.clipboard) navigator.clipboard.writeText(t).catch(() => {}); };
  // The readout's open / compact choice outlives the page (per browser; nothing breaks without storage).
  const READOUT = 'amenui.readout';
  let readout = 0;
  try { const v = localStorage.getItem(READOUT); if (v !== null) { w.set_readout(+v === 2 ? 2 : 0); } } catch (e) {}
  readout = w.readout();
  const keepReadout = () => { const v = w.readout(); if (v !== readout) { readout = v; try { localStorage.setItem(READOUT, String(v)); } catch (e) {} } };
  const after = () => { sync(); syncText(); axUpdate(); clip(); keepReadout(); if (opts.after) opts.after(); kick(); };
  // A change of scheme: from the switch it spreads from where it was pressed, from the system it is a cut.
  if (theme && w.set_scheme) document.addEventListener('amenui:scheme', e => {
    const c = canvas.getBoundingClientRect(), f = e.detail && e.detail.from;
    w.set_scheme(theme(), f ? f[0] - c.left : NaN, f ? f[1] - c.top : NaN);
    after();
  });
  // Mouse, pen and touch come as pointer events (kind 0, 2, 1); the pointer is captured while it is
  // down, so a drag keeps going outside the canvas.
  const kind = e => e.pointerType === 'touch' ? 1 : e.pointerType === 'pen' ? 2 : 0;
  canvas.addEventListener('pointermove', e => {
    for (const c of (e.getCoalescedEvents ? e.getCoalescedEvents() : [e])) w.pointer_move(e.pointerId, kind(e), ...at(c), now(), (c.timeStamp || e.timeStamp) / 1000);
    canvas.style.cursor = w.cursor(); kick();
  });
  canvas.addEventListener('pointerleave', e => { if (kind(e) !== 1) { w.pointer_leave(now()); kick(); } });
  // The page decides where focus goes (the canvas, or the text element when a field is pressed), so the
  // browser's own focus change on press is suppressed. Only the primary button presses.
  canvas.addEventListener('pointerdown', e => {
    e.preventDefault();
    if (kind(e) === 0 && e.button !== 0) return;
    canvas.focus({ preventScroll: true }); canvas.setPointerCapture(e.pointerId);
    const [x, y] = at(e);
    if (kind(e) !== 1) w.pointer_move(e.pointerId, kind(e), x, y, now(), e.timeStamp / 1000);
    w.pointer_down(e.pointerId, kind(e), x, y, e.shiftKey, now(), e.timeStamp / 1000); after();
  });
  canvas.addEventListener('pointerup', e => { if (kind(e) === 0 && e.button !== 0) return; w.pointer_up(e.pointerId, kind(e), ...at(e), now()); after(); });
  canvas.addEventListener('pointercancel', e => { w.pointer_cancel(e.pointerId, now()); after(); });
  canvas.addEventListener('contextmenu', e => e.preventDefault());
  // Wheel deltas in pixels; lines are 40 px (as on the native window), pages the canvas's height. A
  // pinch arrives as a wheel with Ctrl: it is not a scroll, and the page is not zoomed.
  canvas.addEventListener('wheel', e => {
    e.preventDefault();
    if (e.ctrlKey) return;
    const k = e.deltaMode === 1 ? 40 : e.deltaMode === 2 ? canvas.clientHeight : 1;
    w.wheel(...at(e), e.deltaX * k, e.deltaY * k, now()); kick();
  }, { passive: false });
  // Keys go to the canvas only while focus is on it (or on nothing): the page's own links and buttons
  // keep Tab, Enter and the rest.
  const ours = () => { const a = active(); return !a || a === canvas || a === ta || a === document.body || ax.contains(a); };
  (opts.keys || window).addEventListener('keydown', e => {
    // Keys the input method is composing with are its own; keys the page has taken (the search's `/`
    // and ⌘K / Ctrl+K) are not the canvas's.
    if (e.isComposing || e.keyCode === 229 || e.defaultPrevented) return;
    const cmd = e.ctrlKey || e.metaKey;
    if (e.key === 'F2' && !ours()) { e.preventDefault(); w.set_readout(w.readout() === 2 ? 0 : 2); after(); return; }
    if (!ours()) return;
    if (cmd) {
      const k = e.key.toLowerCase();
      // Copy, cut and paste arrive as clipboard events; other shortcuts belong to the browser.
      if (!['a', 'z', 'y'].includes(k)) return;
      if (w.key_mods(e.key, e.shiftKey, true, e.altKey, now())) e.preventDefault();
      after(); return;
    }
    const text = w.text_focused();
    if (e.altKey && !text) return;
    // In a text field characters arrive as input events (so the input method can take them).
    if (text && e.key.length === 1) return;
    if (w.key_mods(e.key, e.shiftKey, false, e.altKey, now()) || e.key === 'Tab') e.preventDefault();
    after();
  });
  ta.addEventListener('input', e => {
    if (e.isComposing) return;
    if (e.inputType === 'insertLineBreak') w.key_mods('Enter', false, false, false, now());
    // A composition's text comes through the composition events.
    else if (e.data && e.inputType !== 'insertCompositionText') w.text_input(e.data, now());
    ta.value = ''; after();
  });
  ta.addEventListener('compositionupdate', e => { w.ime_preedit(e.data || '', now()); after(); });
  ta.addEventListener('compositionend', e => { w.ime_commit(e.data || '', now()); ta.value = ''; after(); });
  document.addEventListener('copy', e => { if (!ours()) return; const t = w.copy(); if (t != null) { e.clipboardData.setData('text/plain', t); e.preventDefault(); } });
  document.addEventListener('cut', e => { if (!ours()) return; const t = w.cut(now()); if (t != null) { e.clipboardData.setData('text/plain', t); e.preventDefault(); after(); } });
  document.addEventListener('paste', e => { if (!ours()) return; const t = e.clipboardData.getData('text/plain'); if (t && w.paste(t, now())) { e.preventDefault(); after(); } });
  (opts.keys || window).addEventListener('keyup', e => { if (ours()) { w.key_up(e.key, now()); after(); } });
  // Reduced motion: the Lab's living pieces and the home page's idle turn rest.
  const rm = window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : null;
  if (rm) { w.set_reduced_motion(rm.matches); if (rm.addEventListener) rm.addEventListener('change', e => { w.set_reduced_motion(e.matches); kick(); }); }
  // No hover (touch): the Lab index runs the cell nearest the centre.
  const hv = window.matchMedia && w.set_hover ? window.matchMedia('(hover: hover)') : null;
  if (hv) { w.set_hover(hv.matches); if (hv.addEventListener) hv.addEventListener('change', e => { w.set_hover(e.matches); kick(); }); }
  // A new size clears the canvas, and the observer runs after this refresh's animation frame and before
  // its paint: the frame is drawn again here, at the new size, so no refresh shows an empty canvas.
  new ResizeObserver(() => {
    fit(); w.resize(canvas.width, canvas.height, dpr); if (opts.onResize) opts.onResize();
    if (raf) { cancelAnimationFrame(raf); raf = 0; }
    loop();
  }).observe(stage);
  kick();
  // What a host can change from outside: the page (a path, a hash name or a look:v3: recipe), the
  // theme and material by name, the language by tag.
  return {
    w, compiled, kick,
    route: h => { const ok = w.route(h); if (ok) w.reset_stage(); w.pictures(); after(); return ok; },
    look: (theme, material) => { w.set_look(theme || '', material || ''); after(); },
    locale: tag => { w.set_locale(tag || ''); after(); },
    focus: () => canvas.focus({ preventScroll: true }),
  };
}

// The site's own pages: the shell (web/shell.html) has started the downloads and set `AMENUI`.
function site() {
  const A = window.AMENUI, D = document, root = D.documentElement;
  const stage = D.getElementById('stage');
  if (!A || !stage) return;
  window.__labMounted = true;
  const full = p => A.base + A.prefix + p;
  let api = null, intro = false;
  const isHome = () => root.classList.contains('home');
  // The home page's stage, laid out by the page: the canvas draws in the same rectangle.
  const place = () => {
    if (!api) return;
    if (!isHome()) { api.w.set_stage(0, 0, 0, 0, -1); return; }
    const c = stage.getBoundingClientRect(), [x0, y0, x1, y1] = A.stage(), cy = A.centre();
    api.w.set_stage(x0 - c.left, y0 - c.top, x1 - c.left, y1 - c.top, cy < 0 ? -1 : cy - c.top);
    api.kick();
  };
  // The rail row: ▸, the four states, the body's form and seed, and the door to the current state's
  // section (`skip ›` while the walk plays, `continue ›` once a person has taken it over).
  const rail = D.querySelector('.rail'), door = rail.querySelector('.door');
  const DOORS = { 1: ['docs/', 'docs →'], 2: ['components/', 'components →'], 3: ['lab/', 'lab →'], 4: ['works/', 'works →'] };
  const NAMES = { 1: 'source', 2: 'interface', 3: 'material', 4: 'body' };
  const INTRO = 'amenui.intro', VERSION = '1';
  const walk = () => api ? api.w.hero_walk() : [0, 0, 1];
  // The rail and the address read the scene's state after every frame, so they name what the stage
  // draws however the walk ends (and whenever a late press lands).
  let railWas = '';
  const followRail = () => { if (!api || !isHome()) return; const k = walk().join() + (intro ? 'i' : ''); if (k === railWas) return; railWas = k; showRail(); keepHash(); };
  const showRail = () => {
    if (!api || !isHome()) return;
    const [playing, seen, at] = walk();
    if (seen) { try { localStorage.setItem(INTRO, VERSION); } catch (e) {} }
    const st = intro ? 1 : at;
    for (const b of rail.querySelectorAll('.st')) b.setAttribute('aria-pressed', String(b.dataset.state === NAMES[st]));
    rail.classList.toggle('body', st === 4 && !intro && !playing);
    rail.querySelector('.seed').textContent = String(api.w.hero_seed()).padStart(2, '0');
    delete door.dataset.skip; delete door.dataset.play;
    if (intro || playing === 1) { door.textContent = 'skip ›'; door.dataset.skip = '1'; door.removeAttribute('href'); }
    else if (playing === 2) { door.textContent = 'continue ›'; door.dataset.play = '1'; door.removeAttribute('href'); }
    else { const [r, t] = DOORS[st]; door.textContent = t; door.dataset.route = r; door.href = full(r); }
  };
  // The state is in the address (`#source`, `#interface`, `#material`, `#body/cloud/08`; none for
  // the defaults), replaced, not added to the history.
  const keepHash = () => {
    if (!api || !isHome() || intro || walk()[0] === 1) return;
    const st = api.w.hero(), want = st ? '#' + st : '';
    if (location.hash !== want && !location.hash.startsWith('#look:')) history.replaceState(history.state, '', location.pathname + want);
  };
  const press = what => { api.w.hero_press(what, performance.now() / 1000); showRail(); keepHash(); api.kick(); };
  const skip = () => { if (intro) { intro = false; api.w.set_hero('', 0); } else press('skip'); try { localStorage.setItem(INTRO, VERSION); } catch (e) {} showRail(); api.kick(); };
  rail.addEventListener('click', e => {
    const b = e.target.closest('button, a');
    if (!b || !api) return;
    if (b.dataset.skip) { e.preventDefault(); skip(); }
    else if (b.dataset.play || b.classList.contains('play')) { e.preventDefault(); press('play'); }
    else if (b.dataset.state) press(b.dataset.state);
    else if (b.dataset.seed) press(+b.dataset.seed > 0 ? 'seed+' : 'seed-');
  });
  // Esc, or a scroll of more than 8 px, skips the walk.
  D.addEventListener('keydown', e => { if (e.key === 'Escape' && api && isHome() && (intro || walk()[0] === 1)) skip(); });
  addEventListener('scroll', () => { if (api && isHome() && scrollY > 8 && (intro || walk()[0] === 1)) skip(); }, { passive: true });
  const routed = (path, hash) => {
    // The docs are HTML pages: a docs address reached inside the app is a navigation to its page.
    if (path.startsWith('docs/')) { location.assign(full(path)); return; }
    A.mark(path);
    if (location.pathname !== full(path)) history.pushState({ path }, '', full(path) + (location.hash.startsWith('#look:') ? location.hash : ''));
    const t = api ? api.w.title() : '';
    D.title = path === '' ? 'amenui' : (t ? t + ' · amenui' : 'amenui');
    place(); showRail();
  };
  // Nothing turns by itself while the tab is hidden or less than half the stage is in view (AA.14).
  let hidden = false, inView = true;
  const away = () => { if (!api || !api.w.hero_press) return; api.w.hero_press(hidden || !inView ? 'hidden' : 'shown', performance.now() / 1000); api.kick(); };
  D.addEventListener('visibilitychange', () => { hidden = D.hidden; away(); });
  const hs = D.getElementById('hstage');
  if (hs && window.IntersectionObserver) new IntersectionObserver(es => { for (const e of es) inView = e.intersectionRatio >= 0.5; away(); }, { threshold: [0, 0.5, 1] }).observe(hs);
  // On a phone the page scrolls under a vertical drag, except one that starts on the body: that turns it.
  D.getElementById('main').addEventListener('touchstart', e => {
    if (!api || !isHome() || e.touches.length !== 1) return;
    const c = api.w.hero_circle(), r = e.target.getBoundingClientRect(), t = e.touches[0];
    if (c.length === 3 && Math.hypot(t.clientX - r.left - c[0], t.clientY - r.top - c[1]) < c[2]) e.preventDefault();
  }, { passive: false });
  const status = D.querySelector('.status'), f2 = status.querySelector('.f2'), quality = status.querySelector('.quality');
  let running = false, lastDrawn = -1, lastLine = '';
  // `F2` opens the full readout (the canvas draws it over the stage's bottom right); while it is open
  // the quality choice is beside the line.
  const keepReadout = () => {
    if (!api) return;
    const open = api.w.readout() === 2, q = api.w.quality();
    f2.setAttribute('aria-expanded', String(open));
    quality.hidden = !open;
    status.classList.toggle('open', open);
    for (const b of quality.querySelectorAll('button')) b.setAttribute('aria-pressed', String(+b.dataset.q === q));
  };
  f2.addEventListener('click', () => { if (!api) return; api.w.set_readout(api.w.readout() === 2 ? 0 : 2); keepReadout(); api.kick(); });
  quality.addEventListener('click', e => { const b = e.target.closest('button'); if (!b || !api) return; api.w.set_quality(+b.dataset.q); keepReadout(); api.kick(); });
  // A piece of Works (M.2): the bar and the status line take the piece's tone, and fade with the
  // piece's chrome after 3 s without input. Reached from another page, the tone changes at 210 ms (M.5).
  const bar = D.querySelector('.bar');
  const tones = [bar.dataset.tone || '', status.dataset.tone || ''];
  let toneTimer = 0, resting = false;
  const keepWork = () => {
    if (!api || !api.w.works_state) return;
    const [t, idle, rest, f] = api.w.works_state();
    root.classList.toggle('idle', !!idle);
    if (!!rest !== resting) { resting = !!rest; keepStatus(); }
    const name = v => v === 1 ? 'light' : v === 2 ? 'dark' : '';
    const want = [name(t), name(f)];
    if (want[0] === tones[0] && want[1] === tones[1]) return;
    tones[0] = want[0]; tones[1] = want[1]; clearTimeout(toneTimer);
    toneTimer = setTimeout(() => { [bar, status].forEach((e, k) => { if (tones[k]) e.dataset.tone = tones[k]; else delete e.dataset.tone; }); }, want[0] ? 210 : 0);
  };
  const keepStatus = () => {
    if (!api || !running) return;
    keepReadout();
    if (resting) { A.line(A.t.rest); return; }
    // The rate is there only while frames are being drawn; a field with nothing to report is left out.
    // The one-line readout (AA.6): `idle` at rest, the rate and a frame's cost while frames are drawn.
    const drawn = api.w.frames()[0], c = api.w.frame_costs();
    const fps = drawn !== lastDrawn && c[3] > 0 ? Math.round(1000 / c[3]) : 0;
    lastDrawn = drawn;
    const stage = api.w.stage();
    const line = `<b>${api.w.backend()}</b> · ` + (fps ? `<b>${fps}</b> ${A.t.fps} · <b>${c[1].toFixed(1)}</b> ms · ` : 'idle · ') + (stage ? `<b>${stage}</b> · ` : '') + `<b>${api.w.glyphs().toLocaleString('en')}</b> ${A.t.glyphs}`;
    if (line !== lastLine) { lastLine = line; A.line(line); }
  };
  const hash = location.hash.startsWith('#look:') ? '' : decodeURIComponent(location.hash.slice(1));
  // The module the shell chose (`A.kind`): the WebGPU-only one where the browser has WebGPU, else the
  // full one, which also draws with WebGL2. If the WebGPU-only one cannot start (no adapter, no device,
  // or anything else), the full one is tried on a fresh canvas before any failure is shown.
  const start = () => mount(stage, {
    // The Lab's index opened at a step (`/lab/#ink`) takes the step with it.
    route: A.route === '' ? '/' : A.route === 'lab/' && hash ? 'lab/#' + hash : A.route,
    load: () => ({ module: A.wasm, glue: A.glue, fonts: A.fonts, texts: A.texts }), faces: A.files.faces, texts: A.files,
    // Before the first frame: the stage, and the state from the address (none: the short path).
    started: w => {
      api = { w, kick: () => {} };
      // How the last frame was drawn, for a log from the console: `AMENUI.framePath()`.
      A.framePath = () => (api && api.w.frame_path ? api.w.frame_path() : '');
      // The material pass made for the page's material forced on (true), off (false) or left to the
      // renderer (null), for an A/B from the console: `AMENUI.specForce(true)`.
      A.specForce = (v) => { if (api && api.w.spec_force) { api.w.spec_force(v === true ? true : v === false ? false : undefined); api.kick(); } };
      if (w.set_status_line) w.set_status_line(true);
      A.line(A.t.starting.replace('{gpu}', navigator.gpu ? 'webgpu' : 'webgl2'));
      // No state in the address: the full walk on a first visit, the short path after (and with reduced
      // motion, which plays no walk).
      if (A.route === '') {
        let seen = false;
        try { seen = localStorage.getItem(INTRO) === VERSION; } catch (e) {}
        const whole = hash === '' && !seen && !matchMedia('(prefers-reduced-motion: reduce)').matches;
        intro = hash === '' && !whole;
        w.set_hero(hash, hash === '' ? (whole ? 2 : 1) : 0);
      }
      place();
    },
    onRoute: (p, h) => routed(p, h),
    onResize: () => { place(); A.draw(); },
    after: () => {
      showRail(); keepHash(); keepReadout(); keepWork();
      // A press that asked for an HTML page (a piece's recipe) goes there.
      const to = api && api.w.take_navigate ? api.w.take_navigate() : '';
      if (to) location.assign(full(to));
    },
    frame: () => { followRail(); keepWork(); },
    onFrame: () => {
      A.done(api.w.backend(), api.w.glyphs());
      if (intro) setTimeout(() => { intro = false; showRail(); keepHash(); }, 1400);
      showRail();
      // The running line is read once, not every second.
      setTimeout(() => { running = true; f2.hidden = false; status.querySelector('.line').setAttribute('aria-live', 'off'); keepStatus(); setInterval(keepStatus, 500); }, 1200);
    },
  });
  const retry = e => {
    if (A.kind !== 'webgpu') throw e;
    let why = '';
    try { why = self.amenuiGlue.webgpu.start_failure(); } catch (_) { /* it did not get that far */ }
    console.warn('the WebGPU-only module could not start (' + (why || 'error') + '); starting the full one', e);
    api = null;
    const c = stage.querySelector('.c');
    // A canvas keeps the first kind of context it gave: the full module asks for its own.
    c.replaceWith(c.cloneNode(false));
    A.load('full');
    return start();
  };
  start().catch(retry).then(a => {
    api = a;
    // The module is in this browser's cache now: a docs page's stage can start as it is scrolled to.
    try { localStorage.setItem('amenui.module', A.files[A.kind === 'webgpu' ? 'wasmGpu' : 'wasm']); } catch (e) {}
    // The search takes a canvas route inside the running app.
    A.go = p => { a.route(p === '' ? '/' : p); };
    a.locale(A.locale);
    if (location.hash.startsWith('#look:')) a.route(location.hash);
    routed(a.w.path(), a.w.hash());
    // The bar's links to canvas pages are taken inside the running app (so the page change plays);
    // with a modifier key or another button they open as links do.
    D.addEventListener('click', e => {
      const l = e.target.closest && e.target.closest('a[data-route]');
      // Links to the docs (HTML pages) navigate as links do.
      if (l && l.dataset.route.startsWith('docs/')) return;
      if (!l || l.dataset.skip || l.dataset.play || e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
      e.preventDefault();
      D.getElementById('sheet').classList.remove('open');
      // The navigation's front starts at the hand (the link's centre for a keyboard activation).
      const r = l.getBoundingClientRect(), c = D.querySelector('#stage .c'), k = c ? c.getBoundingClientRect() : { left: 0, top: 0 };
      const [x, y] = e.detail === 0 ? [r.left + r.width / 2, r.top + r.height / 2] : [e.clientX, e.clientY];
      if (a.w.pressed_at) a.w.pressed_at(x - k.left, y - k.top);
      a.route(l.dataset.route === '' ? '/' : l.dataset.route);
    });
    addEventListener('popstate', () => {
      const p = location.pathname.slice((A.base + A.prefix).length);
      a.route(p === '' ? '/' : p);
    });
  }).catch(e => A.fail(e));
}
site();
