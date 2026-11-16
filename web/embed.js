// <lab-view>: the framework as a custom element, for any page or framework that renders HTML (plain
// HTML, React, Vue, Svelte, Angular). Attributes in, events out, sized by its box (a ResizeObserver).
//
//   <lab-view page="button" theme="dark" material="dot" locale="en" style="height: 480px"></lab-view>
//
// Attributes: `page` (a gallery page's hash, such as `button` or `body`), `recipe` (a `look:v1:…`
// string; wins over the page's own look), `theme` and `material` (names as the render tool takes
// them), `locale` (`zh-Hans`, `en`). Events: `lab-ready` once it draws, `lab-route` with
// `detail.page` whenever the page changes inside it. Methods: `focus()`.
//
// One per document: the wasm module keeps one runtime and one renderer (every export reads that one
// state). A second <lab-view> (or one next to the full page's #stage) is not drawn; it says so in its
// box and sets `data-lab-limit`, and the first keeps working. Several views need one runtime per view
// behind every export (an id on each call), which is not done (DESIGN.md §9).
(() => {
  const CSS = `:host { display: block; position: relative; min-height: 120px; contain: content; }
    .stage { position: absolute; inset: 0; }
    canvas { position: absolute; inset: 0; width: 100%; height: 100%; display: block; touch-action: none; outline: none; user-select: none; -webkit-user-select: none; -webkit-tap-highlight-color: transparent; }
    .msg { position: absolute; inset: 0; display: grid; place-items: center; font: 13px system-ui, sans-serif; color: #888; }
    .a11y { position: absolute; inset: 0; opacity: 0; pointer-events: none; overflow: hidden; }
    .a11y div { position: fixed; overflow: hidden; white-space: nowrap; color: transparent; }
    .ta { position: absolute; left: 0; top: 0; width: 2px; height: 20px; padding: 0; border: 0; margin: 0; resize: none; overflow: hidden; opacity: 0; color: transparent; background: transparent; caret-color: transparent; outline: none; white-space: pre; pointer-events: none; }`;
  class LabView extends HTMLElement {
    static get observedAttributes() { return ['page', 'recipe', 'theme', 'material', 'locale']; }
    connectedCallback() {
      if (this.shadowRoot) return;
      const root = this.attachShadow({ mode: 'open' });
      if (window.__labMounted) {
        root.innerHTML = `<style>${CSS}</style><div class="stage"><div class="msg">one &lt;lab-view&gt; per document: this one is not drawn</div></div>`;
        this.setAttribute('data-lab-limit', 'one-per-document');
        return;
      }
      window.__labMounted = true;
      root.innerHTML = `<style>${CSS}</style><div class="stage"><canvas class="c" tabindex="0"></canvas><textarea class="ta" tabindex="-1" aria-label="text input" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"></textarea><div class="a11y"></div><div class="msg">…</div></div>`;
      const first = this.getAttribute('recipe') || this.getAttribute('page') || 'body';
      this.ready = mount(root.querySelector('.stage'), {
        hash: '#' + first, ownsHash: false, keys: this,
        onRoute: page => this.dispatchEvent(new CustomEvent('lab-route', { detail: { page }, bubbles: true, composed: true })),
      }).then(api => {
        this.api = api;
        this.sync(true);
        this.dispatchEvent(new CustomEvent('lab-ready', { bubbles: true, composed: true }));
        return api;
      });
    }
    attributeChangedCallback() { if (this.api) this.sync(false); }
    sync(first) {
      const a = n => this.getAttribute(n);
      if (!first && a('page')) this.api.route(a('page'));
      if (a('recipe')) this.api.route(a('recipe'));
      if (a('theme') || a('material')) this.api.look(a('theme'), a('material'));
      if (a('locale')) this.api.locale(a('locale'));
    }
    focus() { if (this.api) this.api.focus(); }
  }
  if (!customElements.get('lab-view')) customElements.define('lab-view', LabView);
})();
