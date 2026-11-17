// The docs pages' behaviour, none of which the page needs to be read: copy on code blocks, the
// contents entry in view, the sidebar and menu sheets without a change of address, and the live stage
// (a still until it is pressed; then the module runs in its box).
(() => {
  const D = document, A = window.DOCS || {};
  // Copy: the block's text as written (line numbers are generated content and not part of it).
  for (const b of D.querySelectorAll('.code .copy')) {
    b.hidden = false;
    b.addEventListener('click', () => {
      const t = b.closest('.code').querySelector('code').textContent;
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(t).then(() => { const was = b.textContent; b.textContent = A.copied || 'copied'; setTimeout(() => { b.textContent = was; }, 1500); }, () => {});
    });
  }
  // The sheets: the phone's menu and the sidebar below 1,080 px open by a class, not the address.
  const sheet = D.getElementById('sheet'), menu = D.querySelector('.menu-btn'), side = D.getElementById('side');
  const close = () => { if (sheet) sheet.classList.remove('open'); if (side) side.classList.remove('open'); if (menu) menu.setAttribute('aria-expanded', 'false'); };
  if (menu && sheet) menu.addEventListener('click', e => { e.preventDefault(); const o = !sheet.classList.contains('open'); close(); sheet.classList.toggle('open', o); menu.setAttribute('aria-expanded', String(o)); });
  for (const a of D.querySelectorAll('.path-open')) a.addEventListener('click', e => { e.preventDefault(); close(); side.classList.add('open'); const c = side.querySelector('[aria-current]'); if (c) c.scrollIntoView({ block: 'center' }); });
  for (const a of D.querySelectorAll('.side-close, .sheet .close')) a.addEventListener('click', e => { e.preventDefault(); close(); });
  D.addEventListener('keydown', e => { if (e.key === 'Escape') close(); });
  if (side) side.addEventListener('click', e => { if (e.target.closest('a[href]') && !e.target.closest('.side-close')) close(); });
  // The contents: the last second-level heading above the top fifth of the window is in view.
  const links = [...D.querySelectorAll('.toc ol a')];
  const heads = links.map(a => D.getElementById(decodeURIComponent(a.hash.slice(1)))).filter(Boolean);
  let queued = 0;
  const mark = () => {
    queued = 0;
    let at = -1;
    for (let i = 0; i < heads.length; i++) if (heads[i].getBoundingClientRect().top < innerHeight * 0.2) at = i;
    if (at < 0 && heads.length) at = 0;
    links.forEach((a, i) => a.classList.toggle('on', i === at));
  };
  if (heads.length) { addEventListener('scroll', () => { if (!queued) queued = requestAnimationFrame(mark); }, { passive: true }); mark(); }
  // The live stage: one per page. A press loads the module and the faces and starts the page's route in
  // the box; the caption's state becomes `live`. If the module is already in the cache it starts as the
  // stage comes into view.
  const st = D.querySelector('.stage[data-route]');
  if (!st || !A.files) return;
  const box = st.querySelector('.box'), state = st.querySelector('.state');
  let started = false;
  const start = () => {
    if (started) return; started = true;
    state.textContent = A.starting || '…';
    const s = D.createElement('script');
    s.src = A.files.app;
    s.onload = () => {
      const live = D.createElement('div');
      live.className = 'live';
      live.innerHTML = '<canvas class="c" tabindex="0" aria-label="' + (A.canvas || 'amenui') + '"></canvas><textarea class="ta" tabindex="-1" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" aria-label="text input"></textarea><div class="a11y"></div>';
      box.appendChild(live);
      window.__labMounted = true;
      const get = u => fetch(u).then(r => { if (!r.ok) throw new Error(u + ': HTTP ' + r.status); return r; });
      mount(live, {
        route: st.dataset.route, keys: live,
        load: () => ({ module: get(A.files.wasm), fonts: Promise.all(A.files.fonts.map(f => get(f.url).then(r => r.arrayBuffer()))) }),
        onFrame: () => { try { localStorage.setItem('amenui.module', A.files.wasm); } catch (e) {} const run = box.querySelector('.run'); if (run) run.remove(); st.classList.add('on'); state.textContent = A.live || 'live'; },
      }).then(api => { api.locale('en'); }, e => { state.textContent = A.fail || 'unavailable'; console.error(e); });
    };
    s.onerror = () => { state.textContent = A.fail || 'unavailable'; };
    D.head.appendChild(s);
  };
  st.querySelector('.run').addEventListener('click', start);
  // When this module has run in this browser before (a canvas page or a stage loaded it), it is in the
  // cache: the stage starts as it comes into view.
  let warm = false;
  try { warm = localStorage.getItem('amenui.module') === A.files.wasm; } catch (e) {}
  if (warm && 'IntersectionObserver' in window) new IntersectionObserver((es, o) => { if (es.some(e => e.isIntersecting)) { o.disconnect(); start(); } }, { threshold: 0.5 }).observe(box);
})();
