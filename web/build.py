# Assemble the site from the wasm-bindgen output in dist/pkg: one small HTML page per address (in
# English at the root and in Simplified Chinese under zh-Hans/), the docs as plain HTML pages
# (web/docs.py), the search index, the glue script, the module and the font subsets as separate
# files with content hashes in their names, a 404 page and the embedding example. Written to
# dist/site/.
#
#   BASE=/website/ python3 web/build.py      (BASE: where the site is served; "/" on its own domain)
#
# Needs: fontTools; `wasm-opt` on PATH is used when it makes the module smaller to transfer; RENDER=
# (the render tool) draws the docs stages' stills, written as WebP when Pillow is there.
import gzip, hashlib, html, json, os, pathlib, re, shutil, subprocess, sys
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import docs as docs_html

root = pathlib.Path(__file__).resolve().parent.parent
BASE = os.environ.get('BASE', '/website/')
if not BASE.endswith('/'): BASE += '/'
dist = root / 'dist'
out = dist / 'site'
pkg = dist / 'pkg'

# amenui's source (Cargo's copy, or a local checkout under [patch]): its fonts, texts and version.
meta = json.loads(subprocess.run(['cargo', 'metadata', '--format-version', '1', '--manifest-path', str(root / 'Cargo.toml')], check=True, capture_output=True, text=True).stdout)
amenui = next(p for p in meta['packages'] if p['name'] == 'amenui')
fw = pathlib.Path(amenui['manifest_path']).parent
version = amenui['version']

def front(text):
    """A Markdown file without its leading front matter block (`---` … `---`)."""
    m = re.match(r'\A---\n.*?\n---\n', text, re.S)
    return text[m.end():] if m else text

def catalog(path):
    c = {}
    for line in path.read_text().splitlines():
        if line.startswith('#') or ' = ' not in line: continue
        k, v = line.split(' = ', 1)
        c[k.strip()] = v
    return c

# The pages, from the site itself (paths, sections, titles, the older hash names, the home page's listing).
site_bin = os.environ.get('SITE_BIN')
cmd = [site_bin] if site_bin else ['cargo', 'run', '--quiet', '--profile', os.environ.get('SITE_PROFILE', 'quick'), '--bin', 'site', '--manifest-path', str(root / 'Cargo.toml')]
data = json.loads(subprocess.run(cmd, check=True, capture_output=True, text=True).stdout)
# The site's languages (`website::LOCALES`): English at the root, each other one under its tag.
langs = sorted((l['tag'] for l in data['locales']), key=lambda t: t != 'en')
assert langs[0] == 'en', 'the site has no English'
SHORT = {'en': 'en', 'zh-Hans': '中'}

shutil.rmtree(out, ignore_errors=True)
out.mkdir(parents=True)

def hashed(name, data):
    h = hashlib.sha256(data).hexdigest()[:10]
    stem, dot, ext = name.rpartition('.')
    fn = f'{stem}.{h}.{ext}'
    (out / fn).parent.mkdir(parents=True, exist_ok=True)
    (out / fn).write_bytes(data)
    return fn

def gz(b): return len(gzip.compress(b, 9))

# The module, in two builds of the same code (R.5): the full one (dist/pkg), which draws with WebGPU or
# WebGL2, and the WebGPU-only one (dist/pkg-gpu, `--no-default-features`), which a browser with WebGPU
# downloads. Each is wasm-opt's output when that is smaller to transfer (gzip), else wasm-bindgen's.
opt = shutil.which('wasm-opt')
def module(d, name):
    w = (d / 'website_bg.wasm').read_bytes()
    if opt:
        o = d / 'website_opt.wasm'
        subprocess.run([opt, '-Oz', '--enable-bulk-memory', '--enable-nontrapping-float-to-int', '--enable-sign-ext', '--enable-mutable-globals', '--enable-reference-types', '--enable-multivalue', str(d / 'website_bg.wasm'), '-o', str(o)], check=True)
        ob = o.read_bytes()
        print(f'{name}: wasm-opt -Oz: {len(w)} -> {len(ob)} bytes; gzip {gz(w)} -> {gz(ob)}')
        if gz(ob) < gz(w): w = ob
    return w
pkg_gpu = dist / 'pkg-gpu'
wasm, wasm_gpu = module(pkg, 'full'), module(pkg_gpu, 'webgpu')
wasm_fn, wasm_gpu_fn = hashed('amenui.wasm', wasm), hashed('amenui-webgpu.wasm', wasm_gpu)
# Their sizes as a server compressing them sends them: the boot line's total when there is no
# Content-Length.
wasm_gz, wasm_gpu_gz = gz(wasm), gz(wasm_gpu)
# The size gates (R.5, V.1), in bytes of gzip: the full module (the fall-back) and the WebGPU-only one
# at 2.5 MiB. The full gate is a ratchet on the size the deploy build measures (a local toolchain reads
# about 7 to 12 KB less): with the framework's widget-body cut (amenui 2705fdc) the deploy build measured
# 2,847,812 at website 09214ddb, and the gate was that plus 5,000; it is never set above 2,860,000.
# With the layout's grid cut (amenui 5a98ceb) the deploy build measured 2,773,966 at website bab610ba,
# Weld included, and the gate was that plus 5,000. With the stages' scale and the scroll that moves the
# last frame (amenui ae06c05) the deploy build measured 2,789,005 at website b898df8d, and with cached
# subtrees, cached views and round clips (amenui 80782e6) 2,791,608 at website c05e381f; the gate is that
# plus 5,000. With each glyph's pen told to its pose (amenui e528bf8) and Kern in place of Orbit the
# deploy build measured 2,807,673 at website 85e3251d. With the intro's console and word, one piece from
# 01 to 04, and damage kept as boxes (amenui 1704fa1) the deploy build measured 2,817,229 at website
# eab4174, and with Weld's drag restaged and offsets kept through a view (amenui 8e42221) 2,818,696 at
# website 2bc03124, and with the intro's words flown whole, the word on its own lattice and the stage kept
# through the morphs (amenui 0925272) 2,826,743 at website a88b101; the gate is that plus 5,000. A new piece adds at most 12,000 of its own net code to either module (framework code it is the first to use, such as
# `pair`, is the framework's); more goes back to design with the table by crate.
WASM_GATE, WASM_GPU_GATE = 2_831_743, 2_621_440
assert wasm_gz <= WASM_GATE, f'the full module is {wasm_gz} bytes gzip, over the gate of {WASM_GATE}'
assert wasm_gpu_gz <= WASM_GPU_GATE, f'the WebGPU-only module is {wasm_gpu_gz} bytes gzip, over the gate of {WASM_GPU_GATE}'

# The texts the module does not carry (R.5, `src/texts.rs`): the demos' English, the Lab's Markdown,
# the Lab reader's page and the sources the components' pages show (the framework's examples, its
# widgets and src/morph.rs), each at `texts/<name>`. A page fetches those its route shows beside the
# module (`boot`) and any other the module asks for later; `textsV` changes with any of them.
texts = {'english.txt': (root / 'assets/english.txt').read_text(), 'en/layout.md': (root / 'docs/site/en/layout.md').read_text()}
for p in sorted((root / 'docs/site').glob('*/lab-*.md')): texts[f'{p.parent.name}/{p.name}'] = p.read_text()
for p in sorted((fw / 'examples/components').glob('*.rs')): texts[f'examples/{p.name}'] = p.read_text()
for p in sorted((fw / 'src/widgets').glob('*.rs')): texts[f'widgets/{p.name}'] = p.read_text()
texts['morph.rs'] = (fw / 'src/morph.rs').read_text()
widget_file = {i['name']: i['files'][0].rsplit('/', 1)[-1] for i in json.loads((fw / 'registry.json').read_text())['items'] if i.get('files')}
for name, body in texts.items():
    (out / 'texts' / name).parent.mkdir(parents=True, exist_ok=True)
    (out / 'texts' / name).write_text(body)
texts_v = hashlib.sha256(json.dumps(texts, sort_keys=True).encode()).hexdigest()[:8]
texts_files = {'texts': BASE + 'texts/', 'textsV': texts_v}
def boot_texts(path, lang):
    """The texts a route's first frame shows: the demos' English on every page, and its own."""
    names = ['english.txt']
    parts = path.strip('/').split('/')
    if parts[0] == 'lab' and len(parts) == 2: names.append('en/layout.md' if parts[1] == 'reader' else f'{lang}/lab-{parts[1]}.md')
    if parts[0] == 'components' and len(parts) == 2:
        names.append('examples/' + {'scroll-area': 'scroll'}.get(parts[1], parts[1].replace('-', '_')) + '.rs')
        f = widget_file.get(parts[1]) or widget_file.get(parts[1].replace('-', '_'))
        if f: names.append('widgets/' + f)
    return [n for n in names if n in texts]
print(f'texts: {len(texts)} files, {sum(len(b.encode()) for b in texts.values())} bytes, gzip {sum(gz(b.encode()) for b in texts.values())}')

# The glue. A docs page's stage and the embedded element load the full module, with wasm-bindgen's glue
# for it and the site's script in one file (`app_fn`). The canvas pages load the site's script alone
# (`shell_fn`) and the glue of the module they choose, which sets `amenuiGlue[kind]`.
glue = (pkg / 'website.js').read_text() + '\n' + (root / 'web/app.js').read_text()
app_fn = hashed('app.js', glue.encode())
def own_glue(d, kind):
    g = (d / 'website.js').read_text()
    head = 'let wasm_bindgen = (function(exports) {'
    assert g.startswith(head), f'{d}/website.js does not start as wasm-bindgen --target no-modules writes it'
    return f'(self.amenuiGlue = self.amenuiGlue || {{}}).{kind} = (function(exports) {{' + g[len(head):]
glue_full, glue_gpu = own_glue(pkg, 'full'), own_glue(pkg_gpu, 'webgpu')
glue_full_fn, glue_gpu_fn = hashed('glue.js', glue_full.encode()), hashed('glue-webgpu.js', glue_gpu.encode())
shell_js = (root / 'web/app.js').read_text()
shell_fn = hashed('site.js', shell_js.encode())
embed_fn = hashed('embed.js', (root / 'web/embed.js').read_bytes())

# The faces, cut to the characters the canvas can draw. Everything drawn comes from string literals in
# the site and the framework, the catalogs, the documentation and the sources shown whole. English
# pages get no CJK characters except those in the sources shown as code; Chinese pages get them all.
text = ''.join(p.read_text() for p in [root / 'src/demo.rs', fw / 'src/pictures.rs', fw / 'src/style.rs', fw / 'src/marks.rs', fw / 'src/preset.rs', fw / 'src/mesh.rs', *sorted((fw / 'src/widgets').glob('*.rs')), *sorted((root / 'src/gallery').glob('*.rs')), *sorted((root / 'examples').rglob('*.rs'))])
literals = set(''.join(re.findall(r'"((?:[^"\\]|\\.)*)"', text))) | {chr(c) for c in range(32, 127)}
shown = set()
for p in [fw / 'src/morph.rs', fw / 'AGENTS.md', fw / 'llms.txt', fw / 'CHANGELOG.md', fw / 'registry.json', *sorted((fw / 'src/widgets').glob('*.rs')), root / 'examples/body.rs']: shown |= set(p.read_text())
def cjk(c): return '⺀' <= c <= '鿿' or '豈' <= c <= '﫿' or '＀' <= c <= '￯' or '　' <= c <= '〿'
chars = {t: set() for t in langs}
for p in sorted((root / 'assets/i18n').glob('*.txt')) + sorted((root / 'assets/i18n/web').glob('*.txt')) + sorted((fw / 'assets/i18n').glob('*.txt')):
    for k in chars: chars[k] |= set(p.read_text())
# Of the docs, the canvas draws only the Lab's and the body scene's text (the rest are HTML pages).
for p in sorted((root / 'docs/site').rglob('*.md')):
    if p.stem.startswith(('lab-', 'scene-')):
        for k in chars: chars[k] |= set(front(p.read_text()))
# The shell's own text in the mono face: the bar, the rail row, the boot line, the listing's caret.
chrome = set('█▸‹›→·°|') | set((root / 'web/shell.html').read_text())
for k in chars: chars[k] |= literals | shown | {c for c in chrome if not cjk(c)}
chars['en'] = {c for c in chars['en'] if not cjk(c) or c in shown}
faces = [('sans', 'NotoSansSC-Regular.otf'), ('pixel', 'FusionPixel-12px-Proportional-zh_hans.otf'), ('departure-mono', 'DepartureMono-Regular.otf')]
fonts = {}
for lang, cs in chars.items():
    fonts[lang] = []
    for name, file in faces:
        sub = dist / f'subset-{lang}-{file}'
        if name == 'departure-mono': cs = {c for c in cs if not cjk(c)}
        subprocess.run([sys.executable, '-m', 'fontTools.subset', str(fw / 'assets' / file), '--text=' + ''.join(sorted(cs)), '--output-file=' + str(sub)], check=True, capture_output=True)
        b = sub.read_bytes()
        fn = f'fonts/{name}.' + ('cjk.' if lang != 'en' else '') + hashlib.sha256(b).hexdigest()[:10] + '.otf'
        (out / 'fonts').mkdir(exist_ok=True)
        (out / fn).write_bytes(b)
        fonts[lang].append({'name': name, 'url': BASE + fn, 'size': len(b)})
# The bar's face is the same file in every language.
for lang in langs: fonts[lang][2] = fonts['en'][2]

cats = {t: {**catalog(root / f'assets/i18n/{t}.txt'), **catalog(root / f'assets/i18n/web/{t}.txt')} for t in langs}
def tr(lang, k): return cats[lang].get(k) or cats['en'][k]
template = (root / 'web/shell.html').read_text()
# The chrome the docs pages share with the canvas pages: the rules between /*chrome*/ and /*/chrome*/,
# and the skip link, the bar and the phone's sheet between <!--bar--> and <!--/bar-->.
chrome_css = re.search(r'/\*chrome\*/(.*?)/\*/chrome\*/', template, re.S).group(1)
bar_template = re.search(r'<!--bar-->(.*?)<!--/bar-->', template, re.S).group(1)
# The listing the home page shows while the module loads: the first lines of the text the body is made
# of (`body::Body::listing`), which must be the first lines of examples/body.rs as they are.
listing = data['listing']
file_lines = [l.rstrip() for l in (root / 'examples/body.rs').read_text().split('\n')]
assert listing.split('\n') == file_lines[:len(listing.split('\n'))], 'the listing is not the start of examples/body.rs'

def chrome_vars(lang, path):
    prefix = '' if lang == 'en' else lang + '/'
    href = lambda p: BASE + prefix + p
    # The language switch: each language by its short name, the current one marked.
    lang_switch = '<span>·</span>'.join('<b>%s</b>' % SHORT.get(l, l) if l == lang else '<a href="%s" lang="%s" hreflang="%s">%s</a>' % (BASE + ('' if l == 'en' else l + '/') + path, l, l, SHORT.get(l, l)) for l in langs)
    t = lambda k: tr(lang, k)
    return {
        'lang': lang, 'home_href': href(''), 'href_docs': href('docs/'), 'href_components': href('components/'), 'href_lab': href('lab/'), 'href_works': href('works/'),
        'href_release': href('docs/release-notes/'), 'href_start': href('docs/start/installation/'), 'version': version, 'lang_switch': lang_switch,
        't_sections': t('web.sections'), 't_docs': t('web.docs'), 't_components': t('web.components'), 't_lab': t('web.lab'), 't_works': t('web.works'), 'tone': '', 'foot': '',
        't_search': t('web.search'), 't_menu': t('web.menu'), 't_skip': t('web.skip'), 't_close': t('web.close'),
    }

def fill(s, v): return re.sub(r'\{\{(\w+)\}\}', lambda m: str(v[m.group(1)]), s)

def current(s, section):
    """The current section marked in the bar and the sheet without the script (it keeps it after)."""
    return re.sub(r'(<a [^>]*data-section="%s")' % re.escape(section), r'\1 aria-current="page"', s) if section in ('docs', 'components', 'lab', 'works') else s

# The search (A.7): one script, loaded on first use, and one index per language (written after the
# docs, from them and from the canvas routes); every page carries the few lines that open it.
search_fn = hashed('search.js', (root / 'web/search.js').read_bytes())
SEARCH_TOKEN = '@@SEARCH-INDEX@@'
def search_boot(lang, index_url):
    t = lambda k: tr(lang, k)
    cfg = {'js': BASE + search_fn, 'index': index_url, 'base': BASE + ('' if lang == 'en' else lang + '/'),
           't': {'search': t('web.search'), 'placeholder': t('web.search.placeholder'), 'results': t('web.search.results'), 'none': t('web.search.none'), 'loading': t('web.search.loading'), 'hint': t('web.search.hint'), 'hintDocs': t('web.search.hint.docs'), 'docs': t('web.docs')}}
    return ('<script>window.SEARCH=%s;(()=>{const S=SEARCH,D=document;let p;const open=()=>(p||(p=new Promise((y,n)=>{const s=D.createElement("script");s.src=S.js;s.onload=y;s.onerror=()=>{p=null;n()};D.head.appendChild(s)}))).then(()=>S.open(),()=>{});'
            'D.addEventListener("keydown",e=>{if(e.defaultPrevented||e.isComposing)return;const a=D.activeElement,k=(e.key||"").toLowerCase(),typing=a&&(/^(INPUT|TEXTAREA|SELECT)$/.test(a.tagName)||a.isContentEditable);'
            'if(((e.metaKey||e.ctrlKey)&&k==="k"&&!e.altKey&&!e.shiftKey)||(e.key==="/"&&!e.metaKey&&!e.ctrlKey&&!e.altKey&&!typing)){e.preventDefault();e.stopImmediatePropagation();open()}},true);'
            'for(const b of D.querySelectorAll(".search"))b.addEventListener("click",()=>{const s=D.getElementById("sheet");if(s)s.classList.remove("open");open()})})()</script>\n') % json.dumps(cfg, ensure_ascii=False, separators=(',', ':'))

# The docs pages (web/docs.py) in the chrome above.
docs_css = (root / 'web/docs.css').read_text()
docs_js_fn = hashed('docs.js', (root / 'web/docs.js').read_bytes())
serif = fw / 'assets/SourceSerif4-8pt-subset.ttf'
serif_fn = f'fonts/source-serif.{hashlib.sha256(serif.read_bytes()).hexdigest()[:10]}.ttf'
(out / 'fonts').mkdir(exist_ok=True)
(out / serif_fn).write_bytes(serif.read_bytes())
# The faces the module does not carry, the five Geist Pixel files: fetched when a page uses one
# (`faces_wanted`: the type page, on entering its route), by the framework's name for it.
late = {}
for f in ['GeistPixel-Square.otf', 'GeistPixel-Grid.otf', 'GeistPixel-Circle.otf', 'GeistPixel-Triangle.otf', 'GeistPixel-Line.otf']:
    b = (fw / 'assets' / f).read_bytes()
    fn = f'fonts/{f[:-4].lower()}.{hashlib.sha256(b).hexdigest()[:10]}.otf'
    (out / fn).write_bytes(b)
    late[f] = BASE + fn
mono_url = fonts['en'][2]['url']
faces_css = ('@font-face{font-family:"Departure Mono";src:url(%s) format("opentype");font-display:swap}'
             '@font-face{font-family:"Source Serif 4";src:url(%s) format("truetype");font-display:swap}') % (mono_url, BASE + serif_fn)
def doc_frame(lang, path, title, description, main):
    v = chrome_vars(lang, path)
    bar = current(fill(bar_template, v), 'docs')
    # Without the script the phone's menu is a link to the sheet (`:target`); docs.js opens it in place.
    bar = re.sub(r'<button type="button" class="menu-btn"([^>]*)>(.*?)</button>', r'<a class="menu-btn" href="#sheet"\1>\2</a>', bar)
    alt = ''.join(f'<link rel="alternate" hreflang="{l}" href="{BASE + ("" if l == "en" else l + "/") + path}">' for l in langs)
    t = lambda k: tr(lang, k)
    cfg = {'files': {'wasm': BASE + wasm_fn, 'fonts': fonts[lang], 'faces': late, 'app': BASE + app_fn, **texts_files}, 'copied': t('web.copied'), 'live': t('web.docs.live'), 'starting': t('web.docs.starting'), 'fail': t('web.docs.unavailable'), 'canvas': t('web.canvas')}
    foot = (f'<footer class="foot"><a href="{v["href_release"]}">amenui v{version}</a><a href="https://github.com/amenui-dev/amenui" rel="noopener">github&nbsp;↗</a><span>MIT</span></footer>')
    return ('<!doctype html>\n'
            f'<html lang="{lang}" class="doc">\n<head>\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">\n'
            f'<title>{html.escape(title)}</title>\n<meta name="description" content="{html.escape(description)}">\n<meta name="color-scheme" content="dark light">\n<link rel="icon" href="data:,">\n{alt}\n'
            f'<link rel="preload" href="{mono_url}" as="font" type="font/otf" crossorigin>\n'
            f'<style>{faces_css}{chrome_css}{docs_css}</style>\n<script>document.documentElement.classList.add("js")</script>\n</head>\n<body>\n'
            f'{bar}\n{main}\n{foot}\n'
            f'<script>window.DOCS={json.dumps(cfg, ensure_ascii=False, separators=(",", ":"))}</script>\n<script src="{BASE + docs_js_fn}" defer></script>\n'
            f'{SEARCH_TOKEN}</body>\n</html>\n')

# Build-time stills of the live stages (A.6, F.8): each stage's route alone (`<path>#stage`) drawn by
# the offscreen renderer at the stage's sizes inside its frame (docs.STILLS: the desktop box and a
# phone's, at scale 2), so its first live frame is the same picture. Made when `RENDER` names the
# render tool (it needs a GPU or Mesa's lavapipe); without it a stage is an empty frame until it is run.
stills = {}
render_bin = os.environ.get('RENDER')
if render_bin:
    import concurrent.futures
    shots = dist / 'stills'
    shots.mkdir(exist_ok=True)
    routes = docs_html.stage_routes({'root': root, 'fw': fw, 'data': data})
    # The still's script (R.2 item 8): `#still` draws the stage with a specimen that is closed at rest
    # in the state that shows it (the context menu open, the sheet at its resting detent), and the skip
    # link is focused by a Tab before the capture. The live stage (`#stage`) starts as it is.
    focus = dist / 'still-focus.json'
    focus.write_text(json.dumps([{'at_ms': 800, 'kind': 'key', 'key': 'Tab'}]))
    FOCUSED = {'components/skip-link/'}
    def shoot(job):
        route, kind = job
        w, h = docs_html.STILLS[kind]
        png = shots / (route.strip('/').replace('/', '-') + '-' + kind + '.png')
        events = ['--events', str(focus)] if route in FOCUSED else []
        r = subprocess.run([render_bin, '--page', route + '#still', '--size', f'{w * 2}x{h * 2}', '--scale', '2', '--at', '1500', '--locale', 'en', *events, '--out', str(png)], capture_output=True, text=True)
        return route, kind, png if r.returncode == 0 and png.exists() else None
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 2) as pool:
        for route, kind, png in pool.map(shoot, [(r, k) for r in routes for k in docs_html.STILLS]):
            if not png: print(f'still: {route} ({kind}) could not be drawn'); continue
            try:
                from PIL import Image
                import io
                buf = io.BytesIO()
                Image.open(png).convert('RGB').save(buf, 'WEBP', quality=82, method=6)
                stills.setdefault(route, {})[kind] = hashed('stills/' + png.stem + '.webp', buf.getvalue())
            except ImportError:
                stills.setdefault(route, {})[kind] = hashed('stills/' + png.name, png.read_bytes())
    print(f'stills: {len(stills)} of {len(routes)} stages, {sum((out / f).stat().st_size for v in stills.values() for f in v.values())} bytes')
built = docs_html.build({'root': root, 'fw': fw, 'out': out, 'base': BASE, 'langs': langs, 'cats': cats, 'data': data, 'version': version, 'frame': doc_frame, 'stills': stills})
redirects = built['redirects']

# The search index: every docs page by its sections, and every canvas route by its title. A route that
# has a docs page (a component's specimen, a Lab experiment) is one entry: it opens the route, its docs
# page is its second target, and its text is the docs page's.
reg = json.loads((fw / 'registry.json').read_text())
desc = {i['name'].replace('_', '-'): i for i in reg['items']}
def plain(h): return re.sub(r'\s+', ' ', html.unescape(re.sub(r'<[^>]+>', ' ', h))).strip()
index_urls = {}
for lang in langs:
    entries, docs_of = [], {}
    for w in built['pages']:
        if w['lang'] != lang: continue
        secs, at = [], 0
        body = re.sub(r'<figure class="stage".*?</figure>|<figcaption>.*?</figcaption>', '', w['body'], flags=re.S)
        heads = list(re.finditer(r'<h([234]) id="([^"]+)"><a [^>]*>(.*?)</a></h\1>', body, re.S))
        secs.append(['', '', plain(body[:heads[0].start()] if heads else body)])
        for j, m in enumerate(heads):
            end = heads[j + 1].start() if j + 1 < len(heads) else len(body)
            secs.append([plain(m.group(3)), m.group(2), plain(body[m.end():end])])
        secs = [s for s in secs if s[0] or s[2]]
        if w.get('canvas'):
            # A component's opening is its registry line, whose first sentence search shows.
            if w['url'].startswith('docs/components/') and secs and not secs[0][0]: secs[0][2] = docs_html.first_sentence(secs[0][2])[0]
            docs_of[w['canvas']] = (w, secs); continue
        entries.append([w['url'], w['title'], w['crumb'], w['keywords'], 0, secs])
    names = {'components': tr(lang, 'nav.components'), 'lab': tr(lang, 'nav.lab'), 'works': tr(lang, 'nav.works'), 'home': 'amenui'}
    for p in data['pages']:
        if p['section'] == 'docs' or p.get('alias') and p['path'].startswith('docs'): continue
        sec = p['section']
        title = p['title'][lang] or p['title']['en'] or 'amenui'
        kw = ' '.join([p['hash']] + p['path'].strip('/').split('/'))
        text = ''
        if sec == 'components':
            it = desc.get(p['path'].strip('/').split('/')[-1])
            if it: text = it.get('description_en') if lang == 'en' else it.get('description', '')
        if p['path'] in docs_of:
            w, secs = docs_of.pop(p['path'])
            entries.append([p['path'], title, w['crumb'] if sec == 'components' else (p.get('crumb') or {}).get(lang) or names.get(sec, sec), (kw + ' ' + w['keywords']).strip(), 1, secs, w['url']])
            continue
        if text: text = docs_html.first_sentence(text)[0]
        crumb = (p.get('crumb') or {}).get(lang) or names.get(sec, sec)
        entries.append([p['path'], title, crumb, kw, 1, [['', '', text]] if text else []])
    # A docs page whose route is not a canvas page here keeps its own entry.
    for w, secs in docs_of.values(): entries.append([w['url'], w['title'], w['crumb'], w['keywords'], 0, secs])
    # Two files: the titles and headings, small enough to answer at once, and the sections' text, which
    # follows (the search shows title matches first and adds text matches when it arrives).
    text = json.dumps([[sec[2] for sec in e[5]] for e in entries], ensure_ascii=False, separators=(',', ':')).encode()
    text_url = BASE + hashed(f'search/{lang}.text.json', text)
    heads = json.dumps({'text': text_url, 'pages': [e[:5] + [[[sec[0], sec[1]] for sec in e[5]]] + e[6:] for e in entries]}, ensure_ascii=False, separators=(',', ':')).encode()
    index_urls[lang] = BASE + hashed(f'search/{lang}.json', heads)
    print(f'search index {lang}: {len(entries)} entries; titles and headings gzip {gz(heads)}, text gzip {gz(text)}')
# The docs pages, with their search line now that the index has its name.
doc_sizes = []
for w in built['pages']:
    f = out / ('' if w['lang'] == 'en' else w['lang']) / w['url'] / 'index.html'
    b = f.read_text().replace(SEARCH_TOKEN, search_boot(w['lang'], index_urls[w['lang']])).encode()
    f.write_bytes(b)
    doc_sizes.append((gz(b), w['lang'] + '/' + w['url']))

def page(lang, path, title, section, tone='', foot=''):
    t = lambda k: tr(lang, k)
    prefix = '' if lang == 'en' else lang + '/'
    href = lambda p: BASE + prefix + p
    files = {'wasm': BASE + wasm_fn, 'wasmSize': len(wasm), 'wasmGz': wasm_gz, 'glue': BASE + glue_full_fn,
             'wasmGpu': BASE + wasm_gpu_fn, 'wasmGpuSize': len(wasm_gpu), 'wasmGpuGz': wasm_gpu_gz, 'glueGpu': BASE + glue_gpu_fn, 'fonts': fonts[lang], 'faces': late, **texts_files, 'boot': boot_texts(path, lang)}
    config = {'base': BASE, 'prefix': prefix, 'route': path, 'locale': lang, 'files': files, **({'phone': data['phone']} if path == '' else {}), 't': {'rest': t('works.rest'), 'copied': t('web.copied'), **{k: t('web.' + k) for k in ('fps', 'glyphs', 'compiling', 'starting', 'first-frame')}, 'fail': t('web.fail').replace('{docs}', href('docs/'))}}
    # Older addresses (`#button`) go to the page's path before anything loads; only the root can
    # receive them here.
    redirect = ''
    if path == '':
        redirect = '<script>(()=>{const m=%s,h=decodeURIComponent(location.hash.slice(1));if(h&&!h.startsWith("look:")&&m[h]!==undefined)location.replace(%s+m[h]);})()</script>\n' % (json.dumps(redirects, separators=(',', ':')), json.dumps(BASE))
    classes = ' '.join(c for c in ['home' if path == '' else '', 'work' if path.startswith('works/') and path != 'works/' else '', 'works' if path.startswith('works/') else ''] if c)
    v = chrome_vars(lang, path) | {
        'classes': classes, 'tone': tone, 'foot': foot, 'title': html.escape(title), 'description': html.escape(t('web.description')),
        'redirect': redirect, 'wasm_url': BASE + wasm_fn, 'wasm_gpu_url': BASE + wasm_gpu_fn,
        't_canvas': t('web.canvas'), 't_text_input': t('web.text-input'),
        't_eyebrow': t('web.eyebrow'), 't_lead': t('web.lead'), 't_start': t('web.start'), 't_copy': t('web.copy'), 't_readout': t('web.readout'), 't_quality': t('web.quality'), 't_quality_auto': t('web.quality.auto'),
        'listing': html.escape(listing, quote=False) if path == '' else '', 'mono_url': fonts[lang][2]['url'],
        'config': json.dumps(config, ensure_ascii=False, separators=(',', ':')),
        'search': search_boot(lang, index_urls[lang]),
        'app': f'<script src="{BASE + shell_fn}" defer></script>',
    }
    return current(fill(template, v), section)

# The canvas pages: every address but the docs, which are the HTML pages above.
sizes = []
canvas_pages = [p for p in data['pages'] if p['section'] != 'docs' and not p['path'].startswith('docs/')]
for p in canvas_pages:
    for lang in langs:
        t = p['title'][lang] or p['title']['en']
        title = 'amenui' if p['path'] == '' else f'{t} · amenui'
        d = out / ('' if lang == 'en' else lang) / p['path']
        d.mkdir(parents=True, exist_ok=True)
        b = page(lang, p['path'], title, p['section'], p.get('tone', ''), p.get('foot', '')).encode()
        (d / 'index.html').write_bytes(b)
        sizes.append((gz(b), lang + '/' + p['path']))
# The addresses pages had before they moved (the scenes, now in the Lab): a page that goes on to the
# new one, keeping the state after `#`.
for old, new in data['moved']:
    for lang in langs:
        prefix = '' if lang == 'en' else lang + '/'
        to = BASE + prefix + new
        d = out / prefix / old
        d.mkdir(parents=True, exist_ok=True)
        (d / 'index.html').write_text('<!doctype html>\n<html lang="%s">\n<meta charset="utf-8">\n<title>amenui</title>\n<link rel="canonical" href="%s">\n<meta http-equiv="refresh" content="0; url=%s">\n<script>location.replace(%s + location.hash)</script>\n</html>\n' % (lang, html.escape(to), html.escape(to), json.dumps(to)))
(out / '404.html').write_text(docs_html.not_found({'base': BASE, 'langs': langs, 'cats': cats, 'redirects': redirects}, doc_frame, lambda l: search_boot(l, index_urls[l])))
(out / '.nojekyll').write_text('')
# The embedding example: the glue, the element, and where the files are.
files = {'wasm': BASE + wasm_fn, 'fonts': fonts['en'], **texts_files}
ex = (root / 'web/embed-example.html').read_text().replace('__AMENUI__', f'<script>window.AMENUI_FILES = {json.dumps(files)};</script>\n<script src="{BASE + app_fn}"></script>\n<script src="{BASE + embed_fn}"></script>')
(out / 'embed-example.html').write_text(ex)

home = (out / 'index.html').read_bytes()
# The build check: the listing written into the home page is the body's text, byte for byte.
m = re.search(r'<template id="source">(.*?)</template>', home.decode(), re.S)
assert m and html.unescape(m.group(1)) == listing, "the home page's listing differs from body::Body::listing"
# Every old docs address still has its page, and no docs page is over 60 KB gzip (A.6).
for h, path in redirects.items():
    if path.startswith('docs/'): assert (out / path / 'index.html').exists(), f'#{h} redirects to {path}, which has no page'
assert max(doc_sizes)[0] <= 60 * 1024, f'docs page over 60 KB gzip: {max(doc_sizes)}'
# Every link the site prints resolves (N.2 item 10): each href under the base in every page written,
# and each link the canvas prints (a piece's recipe), in every language.
def resolves(path):
    f = out / path
    return path == '' or (f.is_file() if not path.endswith('/') else (f / 'index.html').is_file()) or (f / 'index.html').is_file()
broken = []
for f in sorted(out.rglob('*.html')):
    for m in re.finditer(r'\shref="([^"]*)"', f.read_text(errors='replace')):
        u = html.unescape(m.group(1)).split('#')[0].split('?')[0]
        if not u.startswith(BASE): continue
        if not resolves(u[len(BASE):]): broken.append(f'{f.relative_to(out)} -> {u}')
for link in data['links']:
    for lang in langs:
        path = ('' if lang == 'en' else lang + '/') + link
        if not resolves(path): broken.append(f'canvas ({lang}) -> {BASE + path}')
assert not broken, f'{len(broken)} links do not resolve:\n' + '\n'.join(broken[:40])

print(f'{len(canvas_pages)} canvas addresses and {len(doc_sizes) // len(langs)} docs pages × {len(langs)} languages; base {BASE}')
print(f'shell (home, en): {len(home)} bytes, gzip {gz(home)}; largest shell gzip {max(sizes)[0]} ({max(sizes)[1]})')
print(f'docs pages gzip: largest {max(doc_sizes)[0]} ({max(doc_sizes)[1]}), median {sorted(doc_sizes)[len(doc_sizes) // 2][0]}')
print(f'wasm, full: {len(wasm)} bytes, gzip {wasm_gz}; glue {len(glue_full)} bytes, gzip {gz(glue_full.encode())}')
print(f'wasm, WebGPU only: {len(wasm_gpu)} bytes, gzip {wasm_gpu_gz}; glue {len(glue_gpu)} bytes, gzip {gz(glue_gpu.encode())}')
print(f'site script: {len(shell_js)} bytes, gzip {gz(shell_js.encode())}; with the full glue (docs stages, the element) {len(glue)} bytes, gzip {gz(glue.encode())}')
for lang in fonts: print(f'fonts {lang}: ' + ', '.join(f'{f["name"]} {f["size"]} (gzip {gz((out / f["url"][len(BASE):]).read_bytes())})' for f in fonts[lang]))
en_fonts = sum(gz((out / f['url'][len(BASE):]).read_bytes()) for f in fonts['en'])
for name, w, g in (('WebGPU', wasm_gpu_gz, glue_gpu), ('WebGL2', wasm_gz, glue_full)):
    print(f'home, English, everything, {name} (gzip): {gz(home) + w + gz(shell_js.encode()) + gz(g.encode()) + en_fonts} bytes')
