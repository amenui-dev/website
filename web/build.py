# Assemble one self-contained page: the wasm module, subsets of the three fonts, and the glue script, inlined.
import base64, pathlib, re, subprocess, sys
root = pathlib.Path(__file__).resolve().parent.parent
# amenui's source (Cargo's copy, or a local checkout under [patch]): its fonts, styles, widgets and texts.
import json
meta = json.loads(subprocess.run(['cargo', 'metadata', '--format-version', '1', '--manifest-path', str(root / 'Cargo.toml')], check=True, capture_output=True, text=True).stdout)
fw = pathlib.Path(next(p['manifest_path'] for p in meta['packages'] if p['name'] == 'amenui')).parent
# Everything drawn comes from string literals in the gallery, the demo, the style names and the widgets
# (whose source the gallery also shows). Any of them
# can be set in any face (the pixel theme sets the whole page in the pixel face), so each face gets all.
text = ''.join(p.read_text() for p in [root / 'src/demo.rs', fw / 'src/pictures.rs', fw / 'src/style.rs', fw / 'src/marks.rs', fw / 'src/preset.rs', fw / 'src/mesh.rs', *sorted((fw / 'src/widgets').glob('*.rs')), *sorted((root / 'src/gallery').glob('*.rs')), *sorted((root / 'examples').rglob('*.rs'))])
chars = set(''.join(re.findall(r'"((?:[^"\\]|\\.)*)"', text))) | {chr(c) for c in range(32, 127)}
# Files the gallery shows whole as source: every character in them, comments included.
for p in [fw / 'src/morph.rs', fw / 'AGENTS.md', fw / 'llms.txt', fw / 'CHANGELOG.md', fw / 'registry.json']: chars |= set(p.read_text())
# Every message of every language (the site's and amenui's assets/i18n), and the documentation pages the site renders.
for p in sorted((root / 'assets/i18n').glob('*.txt')) + sorted((fw / 'assets/i18n').glob('*.txt')) + sorted((root / 'docs').rglob('*.md')) + sorted((fw / 'docs').rglob('*.md')): chars |= set(p.read_text())
subs = []
for name in ['NotoSansSC-Regular.otf', 'FusionPixel-12px-Proportional-zh_hans.otf', 'DepartureMono-Regular.otf']:
    sub = root / ('dist/subset-' + name)
    subprocess.run([sys.executable, '-m', 'fontTools.subset', str(fw / 'assets' / name), '--text=' + ''.join(sorted(chars)), '--output-file=' + str(sub)], check=True)
    subs.append(sub)
wasm = root / 'dist/pkg/website_opt.wasm'
if not wasm.exists(): wasm = root / 'dist/pkg/website_bg.wasm'
b = lambda p: base64.b64encode(p.read_bytes()).decode()
glue = (root / 'dist/pkg/website.js').read_text().replace('</script', '<\\/script')
page = (root / 'web/page.html').read_text().replace('__GLUE__', glue).replace('__WASM__', b(wasm)).replace('__SANS__', b(subs[0])).replace('__PIXEL__', b(subs[1])).replace('__MONO__', b(subs[2]))
embed = '<script>' + (root / 'web/embed.js').read_text() + '</script>'
# The full page defines <amenui-view> too (so the same file can be embedded); the example page carries
# the glue and the element only, without the full page's stage.
script_part = page[page.index('<script>'):].replace('__EMBED__', embed)
page = page.replace('__EMBED__', embed)
(root / 'dist/embed-example.html').write_text((root / 'web/embed-example.html').read_text().replace('__AMENUI__', script_part))
(root / 'dist/artifact.html').write_text(page)
(root / 'dist/index.html').write_text('<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">' + page.replace('<div id="stage">', '</head><body style="margin:0"><div id="stage">', 1) + '</body></html>')
print('wasm', wasm.name, wasm.stat().st_size // 1024, 'KB; fonts', [f.stat().st_size // 1024 for f in subs], 'KB; page', len(page) // 1024, 'KB')
