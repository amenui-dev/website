// The scheme (AA.10), inlined into every page's head so it is set before the first paint: `auto` (the
// system's), `light` or `dark`, as this browser last chose it with the bar's switch (`amenui.theme`).
// The root carries what was chosen (`data-pick`) and what is drawn (`data-theme`); the style sheet
// follows the system by itself when the script does not run. The switch cycles auto · light · dark; a
// change tells the page's canvases (`amenui:scheme`), with the switch's centre when it was pressed
// there, so the change spreads from the switch.
(() => {
  const D = document, r = D.documentElement, KEY = 'amenui.theme', PICKS = ['auto', 'light', 'dark'];
  const sys = matchMedia('(prefers-color-scheme: light)');
  let pick = 'auto';
  try { const v = localStorage.getItem(KEY); if (PICKS.includes(v)) pick = v; } catch (e) {}
  const label = () => { for (const b of D.querySelectorAll('.scheme')) { const v = b.querySelector('.' + pick[0]); if (v) b.setAttribute('aria-label', b.dataset.label + ': ' + v.textContent); } };
  const apply = from => {
    const was = r.dataset.theme, now = pick === 'auto' ? (sys.matches ? 'light' : 'dark') : pick;
    r.dataset.pick = pick;
    r.dataset.theme = now;
    label();
    if (was && was !== now) D.dispatchEvent(new CustomEvent('amenui:scheme', { detail: { from } }));
  };
  apply(null);
  sys.addEventListener('change', () => { if (pick === 'auto') apply(null); });
  D.addEventListener('DOMContentLoaded', label);
  D.addEventListener('click', e => {
    const b = e.target.closest && e.target.closest('.scheme');
    if (!b) return;
    pick = PICKS[(PICKS.indexOf(pick) + 1) % PICKS.length];
    try { localStorage.setItem(KEY, pick); } catch (e) {}
    const c = b.getBoundingClientRect();
    apply([c.left + c.width / 2, c.top + c.height / 2]);
  });
})();
