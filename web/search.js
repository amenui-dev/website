// The search over the whole site (site design A.7): an HTML dialog over any page, the docs and the
// canvas routes in one static index per language, fetched on first open. Loaded by the small script
// every page carries (`SEARCH`), which opens it on the bar's button, `/` and ⌘K / Ctrl+K.
(() => {
  const S = window.SEARCH, D = document;
  const CSS = `.find { position: fixed; inset: 0; z-index: 50; display: flex; justify-content: center; align-items: flex-start; padding: 96px var(--m, 16px) 16px; background: color-mix(in srgb, var(--ground) 82%, transparent); opacity: 0; transition: opacity 120ms linear; }
.find.open { opacity: 1; }
@media (prefers-reduced-motion: reduce) { .find { transition: none; } }
@media (max-width: 719px) { .find { padding-top: 56px; } }
.find .box { width: min(640px, 100%); max-height: calc(100vh - 152px); display: flex; flex-direction: column; background: var(--raised); border: 1px solid var(--hairline); color: var(--text); }
.find .head { display: flex; align-items: center; gap: 12px; height: 48px; padding: 0 16px; border-bottom: 1px solid var(--hairline); font: 11px/11px var(--mono); letter-spacing: .06em; text-transform: uppercase; color: var(--dim); -webkit-font-smoothing: none; }
.find input { flex: 1; min-width: 0; height: 46px; border: 0; outline: 0; background: none; color: var(--ink); font: 17px/1.2 var(--serif, Georgia, serif); padding: 0; }
.find input::placeholder { color: var(--dim); }
.find input::-webkit-search-cancel-button { display: none; }
.find .esc { font: inherit; letter-spacing: inherit; text-transform: inherit; color: var(--dim); background: none; border: 0; padding: 0; height: 44px; cursor: pointer; }
.find .esc:hover { color: var(--ink); }
.find ul { list-style: none; margin: 0; padding: 4px 0; overflow-y: auto; overscroll-behavior: contain; }
.find li a { position: relative; display: block; padding: 10px 16px 10px 32px; color: var(--text); text-decoration: none; }
.find li a[aria-selected="true"] { background: var(--ground); }
.find li a[aria-selected="true"]::before { content: "▸"; position: absolute; left: 16px; top: 10px; color: var(--accent); font: 11px/14px var(--mono); }
.find .k { display: block; font: 11px/14px var(--mono); letter-spacing: .06em; text-transform: uppercase; color: var(--dim); -webkit-font-smoothing: none; }
.find .t { display: block; font: 11px/14px var(--mono); letter-spacing: .06em; text-transform: uppercase; color: var(--ink); margin-top: 4px; -webkit-font-smoothing: none; }
.find li a[aria-selected="true"] .t { color: var(--accent); }
.find .x { display: block; margin-top: 4px; font: 14px/1.45 var(--serif, Georgia, serif); color: var(--text); overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; }
html[lang="zh-Hans"] .find .x, html[lang="zh-Hans"] .find input { font-family: "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Microsoft YaHei", sans-serif; }
.find mark { background: none; color: var(--ink); text-decoration: underline 1px; text-underline-offset: 3px; }
.find .none, .find .foot { margin: 0; padding: 12px 16px; font: 11px/14px var(--mono); letter-spacing: .06em; text-transform: uppercase; color: var(--dim); -webkit-font-smoothing: none; }
.find .foot { border-top: 1px solid var(--hairline); }
@media (max-width: 719px) { .find .foot { display: none; } }`;
  let el, input, list, none, data = null, loading = null, found = [], active = 0, back = null;
  const esc = s => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  const low = s => s.toLowerCase();
  const load = () => loading || (loading = fetch(S.index).then(r => { if (!r.ok) throw new Error(S.index + ': HTTP ' + r.status); return r.json(); }).then(j => {
    // Each entry: [url, title, crumb, keywords, canvas, [[heading, anchor, text], …]], with lowercase copies.
    data = j.map(([url, title, crumb, kw, canvas, secs]) => ({ url, title, crumb, canvas, secs, lt: low(title), lk: low(kw), ls: secs.map(s => [low(s[0]), low(s[2])]) }));
    if (el && el.classList.contains('open')) run();
  }));
  const build = () => {
    const st = D.createElement('style'); st.textContent = CSS; D.head.appendChild(st);
    el = D.createElement('div');
    el.className = 'find'; el.hidden = true;
    el.setAttribute('role', 'dialog'); el.setAttribute('aria-modal', 'true'); el.setAttribute('aria-label', S.t.search);
    el.innerHTML = `<div class="box"><div class="head"><span aria-hidden="true">/</span><input type="search" role="combobox" aria-autocomplete="list" aria-expanded="true" aria-controls="find-list" autocomplete="off" spellcheck="false" placeholder="${esc(S.t.placeholder)}" aria-label="${esc(S.t.search)}"><button type="button" class="esc">esc</button></div><ul id="find-list" role="listbox" aria-label="${esc(S.t.results)}"></ul><p class="none" hidden></p><p class="foot">${esc(S.t.hint)}</p></div>`;
    D.body.appendChild(el);
    input = el.querySelector('input'); list = el.querySelector('ul'); none = el.querySelector('.none');
    input.addEventListener('input', run);
    input.addEventListener('keydown', e => {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); select(active + (e.key === 'ArrowDown' ? 1 : -1)); }
      else if (e.key === 'Enter') { e.preventDefault(); const a = list.children[active] && list.children[active].firstChild; if (a) go(a, e); }
      else if (e.key === 'Escape') { e.preventDefault(); close(); }
      else if (e.key === 'Tab') e.preventDefault();
      e.stopPropagation();
    });
    el.addEventListener('click', e => { if (e.target === el || e.target.closest('.esc')) close(); });
    list.addEventListener('click', e => { const a = e.target.closest('a'); if (a && e.button === 0 && !e.metaKey && !e.ctrlKey && !e.shiftKey) { e.preventDefault(); go(a, e); } });
    list.addEventListener('mousemove', e => { const li = e.target.closest('li'); if (li) select([...list.children].indexOf(li), true); });
  };
  const select = (i, quiet) => {
    const n = list.children.length;
    if (!n) return;
    active = Math.max(0, Math.min(n - 1, i));
    [...list.children].forEach((li, j) => li.firstChild.setAttribute('aria-selected', String(j === active)));
    const a = list.children[active].firstChild;
    input.setAttribute('aria-activedescendant', a.id);
    if (!quiet) a.scrollIntoView({ block: 'nearest' });
  };
  // A canvas route is taken inside the running app when there is one; anything else is a navigation.
  const go = a => {
    const r = a.dataset.route;
    close(true);
    if (r !== undefined && window.AMENUI && window.AMENUI.go) window.AMENUI.go(r);
    else location.href = a.href;
  };
  // Every word must be somewhere in the page; the title counts most, then the keywords, the headings
  // and the text. The section shown is the one that holds most of the words.
  const find = q => {
    const words = low(q).split(/\s+/).filter(Boolean);
    if (!words.length) return [];
    const out = [];
    for (const p of data) {
      let score = 0, ok = true;
      for (const w of words) {
        let s = 0;
        if (p.lt === w) s = 120; else if (p.lt.startsWith(w)) s = 70; else if (p.lt.includes(w)) s = 45;
        if (p.lk.split(' ').includes(w)) s = Math.max(s, 90); else if (p.lk.includes(w)) s = Math.max(s, 25);
        if (!s && p.ls.some(([h, t]) => h.includes(w) || t.includes(w))) s = 4;
        if (!s) { ok = false; break; }
        score += s;
      }
      if (!ok) continue;
      let best = -1, bs = 0;
      // A page found by its title is shown by its opening; otherwise by the section with most words.
      if (words.every(w => p.lt.includes(w)) && p.secs.length && !p.secs[0][1]) { out.push({ p, score, sec: 0 }); continue; }
      p.ls.forEach(([h, t], i) => {
        let s = 0;
        for (const w of words) { if (h.includes(w)) s += 12; if (t.includes(w)) s += 2 + Math.min(4, t.split(w).length - 2); }
        if (s > bs) { bs = s; best = i; }
      });
      out.push({ p, score: score + Math.min(bs, 30) - p.url.length / 1000, sec: best });
    }
    return out.sort((a, b) => b.score - a.score).slice(0, 30);
  };
  const mark = (text, words) => {
    let h = esc(text);
    for (const w of words) { if (w.length < 1) continue; const re = new RegExp(w.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi'); h = h.replace(re, m => '<mark>' + m + '</mark>'); }
    return h;
  };
  const snippet = (text, words) => {
    const l = low(text);
    let at = -1;
    for (const w of words) { const i = l.indexOf(w); if (i >= 0 && (at < 0 || i < at)) at = i; }
    if (at < 0) return text.slice(0, 140);
    const a = Math.max(0, at - 50);
    return (a > 0 ? '…' : '') + text.slice(a, a + 160);
  };
  const run = () => {
    const q = input.value.trim();
    list.innerHTML = '';
    none.hidden = true;
    if (!data) { if (q) { none.textContent = S.t.loading; none.hidden = false; } return; }
    if (!q) return;
    const words = low(q).split(/\s+/).filter(Boolean);
    found = find(q);
    if (!found.length) { none.textContent = S.t.none; none.hidden = false; return; }
    list.innerHTML = found.map(({ p, sec }, i) => {
      const s = sec >= 0 ? p.secs[sec] : null;
      const href = S.base + p.url + (s && s[1] ? '#' + s[1] : '');
      const head = s && s[0] ? ' · ' + s[0] : '';
      const x = s ? snippet(s[2], words) : '';
      return `<li role="presentation"><a id="find-${i}" role="option" href="${esc(href)}"${p.canvas ? ` data-route="${esc(p.url)}"` : ''}><span class="k">${esc(p.crumb)}${esc(head)}</span><span class="t">${mark(p.title, words)}</span>${x ? `<span class="x">${mark(x, words)}</span>` : ''}</a></li>`;
    }).join('');
    select(0);
  };
  const open = () => {
    if (!el) build();
    load().catch(e => { none.textContent = S.t.none; none.hidden = false; console.error(e); });
    if (el.classList.contains('open')) { input.select(); return; }
    back = D.activeElement;
    el.hidden = false;
    void el.offsetWidth; // the closed style first, so the opacity change runs
    el.classList.add('open');
    input.focus(); input.select();
    run();
  };
  const close = leaving => {
    if (!el || el.hidden) return;
    el.classList.remove('open');
    setTimeout(() => { el.hidden = true; }, 130);
    if (!leaving && back && back.focus) back.focus({ preventScroll: true });
  };
  S.open = open;
  S.close = close;
})();
