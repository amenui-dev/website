# Assemble the site from the wasm-bindgen output in dist/pkg: one small HTML page per address (in
# English at the root and in Simplified Chinese under zh-Hans/), the glue script, the module and the
# font subsets as separate files with content hashes in their names, a 404 page and the embedding
# example. Written to dist/site/.
#
#   BASE=/website/ python3 web/build.py      (BASE: where the site is served; "/" on its own domain)
#
# Needs: fontTools; `wasm-opt` on PATH is used when it makes the module smaller to transfer.
import gzip, hashlib, html, json, os, pathlib, re, shutil, subprocess, sys

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
    (out / fn).write_bytes(data)
    return fn

def gz(b): return len(gzip.compress(b, 9))

# The module: wasm-opt's output when it is smaller to transfer (gzip), else wasm-bindgen's.
wasm = (pkg / 'website_bg.wasm').read_bytes()
opt = shutil.which('wasm-opt')
if opt:
    o = pkg / 'website_opt.wasm'
    subprocess.run([opt, '-Oz', '--enable-bulk-memory', '--enable-nontrapping-float-to-int', '--enable-sign-ext', '--enable-mutable-globals', '--enable-reference-types', '--enable-multivalue', str(pkg / 'website_bg.wasm'), '-o', str(o)], check=True)
    ob = o.read_bytes()
    print(f'wasm-opt -Oz: {len(wasm)} -> {len(ob)} bytes; gzip {gz(wasm)} -> {gz(ob)}')
    if gz(ob) < gz(wasm): wasm = ob
wasm_fn = hashed('amenui.wasm', wasm)
# Its size as a server compressing it sends it: the boot line's total when there is no Content-Length.
wasm_gz = gz(wasm)

# The glue: wasm-bindgen's, then the site's.
glue = (pkg / 'website.js').read_text() + '\n' + (root / 'web/app.js').read_text()
app_fn = hashed('app.js', glue.encode())
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
for p in sorted((root / 'assets/i18n').glob('*.txt')) + sorted((fw / 'assets/i18n').glob('*.txt')):
    for k in chars: chars[k] |= set(p.read_text())
for p in sorted((root / 'docs/site').rglob('*.md')):
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

cats = {t: catalog(root / f'assets/i18n/{t}.txt') for t in langs}
template = (root / 'web/shell.html').read_text()
redirects = data['redirects']
# The listing the home page shows while the module loads: the first lines of the text the body is made
# of (`body::Body::listing`), which must be the first lines of examples/body.rs as they are.
listing = data['listing']
file_lines = [l.rstrip() for l in (root / 'examples/body.rs').read_text().split('\n')]
assert listing.split('\n') == file_lines[:len(listing.split('\n'))], 'the listing is not the start of examples/body.rs'

def page(lang, path, title, section, notfound=False):
    t = cats[lang]
    tr = lambda k: t.get(k) or cats['en'][k]
    prefix = '' if lang == 'en' else lang + '/'
    href = lambda p: BASE + prefix + p
    # The language switch: each language by its short name, the current one marked.
    lang_switch = '<span>·</span>'.join('<b>%s</b>' % SHORT.get(l, l) if l == lang else '<a href="%s" lang="%s" hreflang="%s">%s</a>' % (BASE + ('' if l == 'en' else l + '/') + path, l, l, SHORT.get(l, l)) for l in langs)
    files = None if notfound else {'wasm': BASE + wasm_fn, 'wasmSize': len(wasm), 'wasmGz': wasm_gz, 'fonts': fonts[lang]}
    config = {'base': BASE, 'prefix': prefix, 'route': path, 'locale': lang, 'files': files, 't': {'copied': tr('web.copied'), 'fail': tr('web.fail')}}
    # Older addresses (`#button`) go to the page's path before anything loads; only the root and the
    # 404 page can receive them.
    redirect = ''
    if path == '' or notfound:
        redirect = '<script>(()=>{const m=%s,h=decodeURIComponent(location.hash.slice(1));if(h&&!h.startsWith("look:")&&m[h]!==undefined)location.replace(%s+m[h]);})()</script>\n' % (json.dumps(redirects, separators=(',', ':')), json.dumps(BASE))
    classes = ' '.join(c for c in ['home' if path == '' and not notfound else '', 'scenes' if path.startswith('scenes/') and path != 'scenes/' else ''] if c)
    nf = '<main class="nf"><h1>%s</h1><p>%s</p></main>\n' % (html.escape(tr('web.not-found')), html.escape(tr('web.not-found-text'))) if notfound else ''
    v = {
        'lang': lang, 'classes': classes, 'title': html.escape(title), 'description': html.escape(tr('web.description')),
        'redirect': redirect, 'wasm_url': BASE + wasm_fn, 'home_href': href(''),
        'href_docs': href('docs/'), 'href_components': href('components/'), 'href_lab': href('lab/'), 'href_scenes': href('scenes/'),
        'href_release': href('docs/release-notes/'), 'href_start': href('docs/start/installation/'), 'version': version, 'lang_switch': lang_switch,
        't_sections': tr('web.sections'), 't_docs': tr('web.docs'), 't_components': tr('web.components'), 't_lab': tr('web.lab'), 't_scenes': tr('web.scenes'),
        't_search': tr('web.search'), 't_menu': tr('web.menu'), 't_canvas': tr('web.canvas'), 't_text_input': tr('web.text-input'),
        't_eyebrow': tr('web.eyebrow'), 't_lead': tr('web.lead'), 't_start': tr('web.start'), 't_copy': tr('web.copy'), 't_skip': tr('web.skip'),
        'listing': html.escape(listing, quote=False) if path == '' and not notfound else '', 'notfound': nf, 'mono_url': fonts[lang][2]['url'],
        'config': json.dumps(config, ensure_ascii=False, separators=(',', ':')),
        'app': '' if notfound else f'<script src="{BASE + app_fn}" defer></script>',
    }
    s = re.sub(r'\{\{(\w+)\}\}', lambda m: str(v[m.group(1)]), template)
    if notfound:
        s = s.replace(f'<link rel="preload" href="{BASE + wasm_fn}" as="fetch" crossorigin>\n', '').replace(f'<link rel="preload" href="{fonts[lang][2]["url"]}" as="fetch" crossorigin>\n', '')
    return s

sizes = []
for p in data['pages']:
    for lang in langs:
        t = p['title'][lang] or p['title']['en']
        title = 'amenui' if p['path'] == '' else f'{t} · amenui'
        d = out / ('' if lang == 'en' else lang) / p['path']
        d.mkdir(parents=True, exist_ok=True)
        b = page(lang, p['path'], title, p['section']).encode()
        (d / 'index.html').write_bytes(b)
        sizes.append((gz(b), lang + '/' + p['path']))
(out / '404.html').write_text(page('en', '', 'Not found · amenui', 'home', notfound=True))
(out / '.nojekyll').write_text('')
# The embedding example: the glue, the element, and where the files are.
files = {'wasm': BASE + wasm_fn, 'fonts': fonts['en']}
ex = (root / 'web/embed-example.html').read_text().replace('__AMENUI__', f'<script>window.AMENUI_FILES = {json.dumps(files)};</script>\n<script src="{BASE + app_fn}"></script>\n<script src="{BASE + embed_fn}"></script>')
(out / 'embed-example.html').write_text(ex)

home = (out / 'index.html').read_bytes()
# The build check: the listing written into the home page is the body's text, byte for byte.
m = re.search(r'<template id="source">(.*?)</template>', home.decode(), re.S)
assert m and html.unescape(m.group(1)) == listing, "the home page's listing differs from body::Body::listing"

print(f'{len(data["pages"])} addresses × {len(langs)} languages; base {BASE}')
print(f'shell (home, en): {len(home)} bytes, gzip {gz(home)}; largest shell gzip {max(sizes)[0]} ({max(sizes)[1]})')
print(f'wasm {len(wasm)} bytes, gzip {gz(wasm)}; glue {len(glue)} bytes, gzip {gz(glue.encode())}')
for lang in fonts: print(f'fonts {lang}: ' + ', '.join(f'{f["name"]} {f["size"]} (gzip {gz((out / f["url"][len(BASE):]).read_bytes())})' for f in fonts[lang]))
en_total = gz(home) + gz(wasm) + gz(glue.encode()) + sum(gz((out / f['url'][len(BASE):]).read_bytes()) for f in fonts['en'])
print(f'home, English, everything (gzip): {en_total} bytes')
