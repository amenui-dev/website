# The docs as plain HTML pages (site design A.4, A.6, D.5): one page per document, in English at the
# root and in Simplified Chinese under zh-Hans/, written at build time from
#
#   docs/site/{en,zh-Hans}/*.md          the site's own pages (start, guides, concepts, reference, and
#                                        the Lab's and the body scene's recipes)
#   amenui: docs/**/*.md                 grouped by their front matter's kind (docs/README.md, the
#                                        repository's own index, is the docs index here); DESIGN.md
#                                        is cut into one page per section
#   amenui: registry.json, widgets,      one page per component: title, description, the live stage,
#           examples/components/*.rs     installation, usage, examples, API, keyboard
#   amenui: AGENTS.md, llms.txt,         reference and release notes; also served as they are at the
#           CHANGELOG.md, registry.json  site's root
#
# No page is shorter than MIN_WORDS of body (code counted): short Concepts pages are one page with a
# section each, a short section of DESIGN.md joins its neighbour. Pages in English only take their
# Simplified Chinese titles from web/docs-titles.txt.
#
# `build(ctx)` writes the pages and returns what the search index needs (`web/build.py` calls it);
# `not_found(ctx)` is the 404 page.
import html, json, pathlib, re
from md import Doc, attr, code_block, esc, inline, slug

MIN_WORDS = 120

# The tree (A.4): groups in order, each a list of pages; the order here is the sidebar's.
GROUPS = [('start', 'nav.start'), ('guides', 'nav.guides'), ('concepts', 'nav.concepts'), ('components', 'nav.components'), ('reference', 'nav.reference'), ('release-notes', 'nav.release-notes')]
SITE = {
    'start': ['introduction', 'installation', 'first-view', 'project-structure'],
    'guides': ['forms', 'theming', 'i18n', 'rtl', 'accessibility', 'testing', 'performance'],
    'concepts': ['tree-and-messages', 'layout', 'text', 'materials', 'ripples', 'transforms', 'presets', 'annotation', 'three-d', 'particles'],
    'reference': ['cli', 'registry-schema'],
}
COMPONENT_GROUPS = [('inputs', 'nav.group.inputs'), ('display', 'nav.group.display'), ('feedback', 'nav.group.feedback'), ('navigation', 'nav.group.navigation'), ('layout', 'nav.group.layout'), ('data', 'nav.group.data')]
GITHUB = {'site': 'https://github.com/amenui-dev/website', 'fw': 'https://github.com/amenui-dev/amenui'}


def front(text):
    """A Markdown file's front matter `kind`, and the text after it."""
    m = re.match(r'\A---\n(.*?)\n---\n', text, re.S)
    if not m: return None, text
    k = re.search(r'^kind:\s*(\S+)', m.group(1), re.M)
    return (k.group(1) if k else None), text[m.end():]


def split_title(md):
    """The first `# ` heading and the text without it."""
    m = re.search(r'^# (.+)$', md, re.M)
    if not m: return None, md
    return m.group(1).strip(), md[:m.start()] + md[m.end():]


def plain(h):
    return re.sub(r'\s+', ' ', html.unescape(re.sub(r'<[^>]+>', ' ', h))).strip()


def words(md):
    """A page's length: the words of its text and its code, as rendered."""
    return len(plain(Doc(md).html).split())


def label(title):
    """A title as plain text, for search and the document's title (code spans unmarked)."""
    return title.replace('`', '')


FILE_NAME = re.compile(r'(?<![\w./-])((?:[\w-]+/)*[\w-]+\.(?:md|txt|rs|toml|json|js|wgsl))(?![\w])')


def label_html(title):
    """A title for a label (the sidebar, the index, the page-end cells): escaped, its code spans and
    the file names in it (`AGENTS.md`, `llms.txt`) as `<code>`, which keep their case where labels
    are set in capitals."""
    plain = lambda x: FILE_NAME.sub(lambda m: f'<code>{m.group(1)}</code>', esc(x))
    return ''.join(f'<code>{esc(x)}</code>' if i % 2 else plain(x) for i, x in enumerate(title.split('`')))


def has_cjk(s):
    return any('\u2e80' <= c <= '\u9fff' or '\uf900' <= c <= '\ufaff' or '\uff00' <= c <= '\uffef' for c in s)


def first_sentence(s):
    """A registry line's first sentence (I.3 ruling 5): up to the first full stop that ends one."""
    m = re.match(r'^(.+?(?:[.!?](?=\s|$)|[。！？]))', s.strip(), re.S)
    return (m.group(1) if m else s.strip()), (s.strip()[m.end():].strip() if m else '')


def shift_md(md, by):
    """Markdown with every heading `by` levels deeper (outside code fences)."""
    out, fence = [], False
    for l in md.split('\n'):
        if l.lstrip().startswith(('```', '~~~')): fence = not fence
        out.append('#' * by + l if not fence and re.match(r'^#{1,5} ', l) else l)
    return '\n'.join(out)


def zh_titles(root):
    t = {}
    for l in (root / 'web/docs-titles.txt').read_text().splitlines():
        if l.startswith('#') or ' | ' not in l: continue
        en, zh = l.split(' | ', 1)
        t[en.strip()] = zh.strip()
    return t


class Page:
    def __init__(self, key, group, path, **kw):
        self.key, self.group, self.path = key, group, path
        self.sub = kw.get('sub')            # (id, title key or text) of a sub-group in the sidebar
        self.md = kw.get('md', {})           # language -> Markdown (front matter and H1 removed)
        self.title = kw.get('title', {})     # language -> title text
        self.content_lang = kw.get('content_lang', {})
        self.src = kw.get('src')             # (repo, path) of the source file, for links and `edit`
        self.hidden = kw.get('hidden', False)
        self.canvas = kw.get('canvas')       # the canvas route this page is the text of
        self.stage = kw.get('stage')         # the route its live stage runs
        self.html = kw.get('html')           # language -> ready HTML (components), instead of md
        self.raw = kw.get('raw', {})         # language -> the raw text served beside the page
        self.keywords = kw.get('keywords', '')
        self.hash = kw.get('hash')           # the older hash name (`docs-layout`) that redirects here
        self.num = ''
        self.out = {}                        # language -> (body html, headings, lead)


def build(ctx):
    root, fw, BASE, langs, cats = ctx['root'], ctx['fw'], ctx['base'], ctx['langs'], ctx['cats']
    canvas_paths = {p['path'] for p in ctx['data']['pages']}
    def tr(lang, k):
        return cats[lang].get(k) or cats['en'].get(k) or k
    zh, untitled = zh_titles(root), set()
    def titled(en):
        """An English-only page's title in each language."""
        out = {}
        for l in langs:
            if l == 'en' or has_cjk(en) or en in ('AGENTS.md', 'llms.txt'): out[l] = en
            elif l == 'zh-Hans' and en in zh: out[l] = zh[en]
            else: out[l] = en; untitled.add(en)
        return out
    pages = []

    # The site's pages.
    for g, slugs in SITE.items():
        for s in slugs:
            md, title, cl = {}, {}, {}
            for lang in langs:
                f = root / 'docs/site' / lang / f'{s}.md'
                if not f.exists(): f, c = root / 'docs/site/en' / f'{s}.md', 'en'
                else: c = lang
                t, body = split_title(front(f.read_text())[1])
                md[lang], title[lang], cl[lang] = body, t or s, c
            pages.append(Page(s, g, f'docs/{g}/{s}/', md=md, title=title, content_lang=cl, src=('site', f'docs/site/{{lang}}/{s}.md'), hash=f'docs-{s}'))
    # The Lab's and the body scene's recipes: not in the sidebar; each is the text under its stage.
    for f in sorted((root / 'docs/site/en').glob('*.md')):
        name = f.stem
        if not name.startswith(('lab-', 'scene-', 'work-')): continue
        if any(name in v for v in SITE.values()): continue
        short = name.split('-', 1)[1]
        md, title, cl = {}, {}, {}
        for lang in langs:
            ff = root / 'docs/site' / lang / f'{name}.md'
            c = lang if ff.exists() else 'en'
            if not ff.exists(): ff = f
            t, body = split_title(front(ff.read_text())[1])
            md[lang], title[lang], cl[lang] = body, t or short, c
        canvas = f'lab/{short}/' if name.startswith('lab-') else f'works/{short}/' if name.startswith('work-') else f'lab/scenes/{short}/'
        if canvas not in canvas_paths: canvas = None
        pages.append(Page(f'recipe-{short}', 'recipes', f'docs/recipes/{short}/', md=md, title=title, content_lang=cl, src=('site', f'docs/site/{{lang}}/{name}.md'), hidden=True, canvas=canvas))
    # Concepts pages under MIN_WORDS: one page, each a section with the page's name as its anchor, at
    # the place of the first of them. Their old addresses lead to their sections.
    short = [p for p in pages if p.group == 'concepts' and words(p.md['en']) < MIN_WORDS]
    merged = None
    if len(short) > 1:
        merged = Page('concepts-overview', 'concepts', 'docs/concepts/overview/', title={l: tr(l, 'web.docs.overview') for l in langs}, content_lang={l: l for l in langs})
        merged.parts = short
        merged.md = {l: '\n\n'.join(f'## {q.title[l]}\n' + shift_md(q.md[l], 1) for q in short) for l in langs}
        pages.insert(pages.index(short[0]), merged)
        pages = [p for p in pages if p not in short]
    placed = {s for v in SITE.values() for s in v}
    for f in (root / 'docs/site/en').glob('*.md'):
        assert f.stem in placed or f.stem.startswith(('lab-', 'scene-')), f'docs/site/en/{f.name} has no place in the docs tree (web/docs.py SITE)'

    # amenui's docs, by kind.
    def fw_page(rel, group, path, key, sub=None, title=None):
        kind, text = front((fw / rel).read_text())
        t, body = split_title(text)
        return Page(key, group, path, sub=sub, md={l: body for l in langs}, title=titled(title or t or key), content_lang={l: 'en' for l in langs}, src=('fw', rel))
    fwdocs = sorted(p.relative_to(fw).as_posix() for p in (fw / 'docs').rglob('*.md'))
    kinds = {r: front((fw / r).read_text())[0] for r in fwdocs}
    for r, k in kinds.items(): assert k in ('readme', 'howto', 'reference', 'explanation'), f'amenui {r}: front matter kind {k!r}'
    for r in fwdocs:
        if kinds[r] == 'howto' and r.startswith('docs/guides/'):
            s = pathlib.PurePosixPath(r).stem
            pages.append(fw_page(r, 'guides', f'docs/guides/{s}/', s))
    for r in fwdocs:
        if kinds[r] == 'howto' and r.startswith('docs/recipes/'):
            s = pathlib.PurePosixPath(r).stem
            pages.append(fw_page(r, 'guides', f'docs/guides/recipes/{s}/', f'fw-recipe-{s}', sub=('recipes', 'docs.recipes')))
    # DESIGN.md (the explanation): one page per section, the first one its introduction and terms.
    design = []
    for r in fwdocs:
        if kinds[r] == 'explanation': design += design_pages(fw, r, langs, titled)
    pages += [p for p in design]
    # Components.
    reg = json.loads((fw / 'registry.json').read_text())
    items = [i for i in reg['items'] if i['name'] != 'base']
    by_name = {i['name']: i for i in reg['items']}
    for gid, _ in COMPONENT_GROUPS:
        for it in items:
            if it.get('group') != gid: continue
            h = it['name'].replace('_', '-')
            canvas = f'components/{h}/'
            pages.append(Page(f'component-{h}', 'components', f'docs/components/{h}/', sub=(gid, dict(COMPONENT_GROUPS)[gid]),
                title={l: (it['name'].replace('_', ' ') if l == 'en' else it.get('title') or it['name']) for l in langs}, content_lang={l: l for l in langs},
                src=('fw', it['files'][0]) if it.get('files') else None, canvas=canvas if canvas in canvas_paths else None, stage=canvas if canvas in canvas_paths else None,
                keywords=' '.join([it['name'], h] + it.get('functions', []))))
    for it in items: assert it.get('group') in dict(COMPONENT_GROUPS), f'registry: {it["name"]} has no known group'
    # Reference and release notes.
    def rootfile(rel, key, path, group, title_key, as_text=False):
        text = (fw / rel).read_text()
        t, body = split_title(text)
        return Page(key, group, path, md={l: body for l in langs}, title={l: (t if not as_text else None) or tr(l, title_key) for l in langs}, content_lang={l: 'en' for l in langs}, src=('fw', rel))
    ref = [p for p in pages if p.group == 'reference']
    pages = [p for p in pages if p.group != 'reference']
    agents = rootfile('AGENTS.md', 'agents', 'docs/reference/agents/', 'reference', 'docs.ai.agents'); agents.hash = 'ai-agents'
    agents.title = {l: 'AGENTS.md' for l in langs}
    llms = rootfile('llms.txt', 'llms', 'docs/reference/llms/', 'reference', 'docs.ai.llms'); llms.hash = 'ai-llms'
    llms.title = {l: 'llms.txt' for l in langs}
    registry = Page('registry', 'reference', 'docs/reference/registry/', title={l: tr(l, 'docs.ai.registry') for l in langs}, content_lang={l: l for l in langs}, src=('fw', 'registry.json'), hash='ai-registry')
    arch = [fw_page(r, 'reference', f'docs/reference/{pathlib.PurePosixPath(r).stem}/', pathlib.PurePosixPath(r).stem) for r in fwdocs if kinds[r] == 'reference']
    pages += ref + [registry, agents, llms] + arch
    changelog = rootfile('CHANGELOG.md', 'changelog', 'docs/release-notes/', 'release-notes', 'docs.changelog'); changelog.hash = 'changelog'
    changelog.title = {l: tr(l, 'nav.release-notes') for l in langs}
    pages.append(changelog)

    # Order: by group, then as listed; sub-groups after the group's own pages; numbers from the order.
    order = {g: i for i, (g, _) in enumerate(GROUPS)}
    seq = {id(p): i for i, p in enumerate(pages)}
    visible = sorted([p for p in pages if not p.hidden], key=lambda p: (order[p.group], 0 if p.group == 'components' else (1 if p.sub else 0), seq[id(p)]))
    for gi, (g, _) in enumerate(GROUPS):
        for j, p in enumerate([p for p in visible if p.group == g]): p.num = f'{gi + 1:02d}.{j + 1}'
    hidden = [p for p in pages if p.hidden]
    allp = visible + hidden
    paths = [p.path for p in allp]
    assert len(set(paths)) == len(paths), 'two docs pages at one path'

    # Where links go: a source file (by repository and path) to its page.
    files = {}
    for p in allp:
        if p.src:
            repo, rel = p.src
            for lang in langs: files[(repo, rel.replace('{lang}', lang))] = p
    files[('fw', 'docs/recipes')] = next(p for p in visible if p.sub and p.sub[0] == 'recipes')
    # amenui's own docs index is this site's: links to it go to /docs/.
    class Index: path = 'docs/'
    files[('fw', 'docs/README.md')] = files[('fw', 'docs')] = Index
    # A merged Concepts page's sections: a link to one of its files goes to its section.
    anchors = {}
    for q in (merged.parts if merged else []):
        for lang in langs:
            files[('site', q.src[1].replace('{lang}', lang))] = merged
            anchors[('site', q.src[1].replace('{lang}', lang))] = q.key
    redirects = dict(ctx['data']['redirects'])
    for p in allp + (merged.parts if merged else []):
        if p.hash: redirects[p.hash] = p.path
    moved = {q.hash: merged.path + '#' + q.key for q in (merged.parts if merged else [])}
    def to(q, key, frag):
        return q.path + ('#' + frag if frag else '#' + anchors[key] if key in anchors else '')
    # Section numbers of DESIGN.md (§1.8.2) to their pages and anchors.
    sections = {}
    for p in design:
        for num, anchor in p.sections: sections[num] = (p, anchor)

    def resolver(p, lang):
        prefix = BASE + ('' if lang == 'en' else lang + '/')
        def link(url):
            gh = re.match(r'^https://github\.com/amenui-dev/(website|amenui)/(?:blob|tree)/main/([^#]+)(#.*)?$', url)
            if gh:
                key = ('site' if gh.group(1) == 'website' else 'fw', re.sub(r'^docs/site/[^/]+/', 'docs/site/' + lang + '/', gh.group(2)) if gh.group(1) == 'website' else gh.group(2))
                q = files.get(key)
                if q: return (prefix + to(q, key, (gh.group(3) or '')[1:]), False)
            if re.match(r'^[a-z]+:', url):
                site = re.match(r'^https://amenui-dev\.github\.io/website/(.*)$', url)
                return (prefix + site.group(1), False) if site else (url, True)
            if url.startswith('/'): return (url, False)
            if url.startswith('#'):
                h = url[1:]
                if h in moved: return (prefix + moved[h], False)
                if h in redirects: return (prefix + redirects[h], False)
                return ('@@#' + h + '@@', False)
            path, _, frag = url.partition('#')
            repo, rel = p.src if p.src else ('site', '')
            rel = rel.replace('{lang}', lang)
            base = pathlib.PurePosixPath(rel).parent
            m = re.match(r'^md/[^/]+/(.+)\.md$', path)
            if m and m.group(1) in moved: return (prefix + moved[m.group(1)], False)
            if m and m.group(1) in redirects: return (prefix + redirects[m.group(1)], False)
            target = norm(base / path)
            key = (repo, target) if (repo, target) in files else (repo, target.rstrip('/'))
            q = files.get(key)
            if q: return (prefix + to(q, key, frag), False)
            kind = 'tree' if path.endswith('/') else 'blob'
            return (f'{GITHUB[repo]}/{kind}/main/{target}' + ('#' + frag if frag else ''), True)
        return link

    # Render every page's text.
    for p in allp:
        for lang in langs:
            t = lambda k, lang=lang: tr(lang, k)
            if p.key == 'registry': body = registry_md(lang, tr, items, ctx)
            elif p.group == 'components': body = None
            else: body = p.md[lang]
            if body is None:
                html_, heads = component_html(p, lang, tr, by_name, fw, resolver(p, lang))
                p.out[lang] = (html_, heads, '')
                continue
            if p is merged:
                # Each part a section under its title, its own headings one level down; its links
                # resolved from its own file.
                seen, parts, heads = {q.key for q in p.parts}, [], []
                for q in p.parts:
                    d = Doc(q.md[lang], resolver(q, lang), t, shift=1, seen=seen)
                    sec = f'<h2 id="{q.key}"><a href="#{q.key}">{esc(q.title[lang])}</a></h2>\n{d.html}'
                    parts.append(sec if q.content_lang[lang] == lang else f'<section lang="{q.content_lang[lang]}">{sec}</section>')
                    heads += [(2, esc(q.title[lang]), q.key)] + [(lv, txt, hid) for lv, txt, hid, _ in d.headings]
                p.out[lang] = ('\n'.join(parts), heads, '')
                p.ids = getattr(p, 'ids', {}); p.ids[lang] = {hid for _, _, hid in heads}
                continue
            d = Doc(body, resolver(p, lang), t, shift=getattr(p, 'shift', 0))
            h = d.html
            # The first paragraph is the lead, in the ink colour (D.5).
            h = h.replace('<p>', '<p class="lead">', 1) if h.lstrip().startswith('<p>') else h
            p.out[lang] = (h, [(lv, txt, hid) for lv, txt, hid, _ in d.headings], '')
            p.ids = getattr(p, 'ids', {}); p.ids[lang] = {hid for _, _, hid, _ in d.headings}
    # In-page anchors (`#rows-and-columns`), and across DESIGN.md's pages; `DESIGN.md §1.8.2` as links.
    design_ids = {}
    for p in design:
        for hid in p.ids.get('en', ()): design_ids.setdefault(hid, p)
    for p in allp:
        for lang in langs:
            h, heads, lead = p.out[lang]
            prefix = BASE + ('' if lang == 'en' else lang + '/')
            def anchor(m):
                a = m.group(1)
                if a in getattr(p, 'ids', {}).get(lang, ()): return '#' + a
                q = design_ids.get(a) if p in design or p.src and p.src[0] == 'fw' else None
                return prefix + q.path + '#' + a if q else '#' + a
            h = re.sub(r'@@#([^@]*)@@', anchor, h)
            h = section_links(h, sections, prefix, p)
            p.out[lang] = (h, heads, lead)

    nav = {'visible': visible, 'hidden': hidden, 'groups': GROUPS}
    written = []
    for p in allp:
        for lang in langs:
            written.append(write_page(ctx, p, lang, nav, tr))
    index = write_index(ctx, nav, tr)
    # Old addresses: a page that goes on. The merged Concepts pages lead to their sections, the page
    # that was amenui's docs index to this one.
    gone = [(q.path, q.title, moved[q.hash]) for q in (merged.parts if merged else [])] + [('docs/start/documentation/', {l: tr(l, 'web.docs.title') for l in langs}, 'docs/')]
    for path, title, to_path in gone:
        for lang in langs:
            prefix = '' if lang == 'en' else lang + '/'
            to_url = BASE + prefix + to_path
            d = ctx['out'] / prefix / path
            d.mkdir(parents=True, exist_ok=True)
            (d / 'index.html').write_text(f'<!doctype html>\n<html lang="{lang}"><meta charset="utf-8"><title>{esc(title[lang])} · amenui</title><link rel="canonical" href="{attr(to_url)}">'
                                          f'<meta http-equiv="refresh" content="0; url={attr(to_url)}"><meta name="robots" content="noindex"><a href="{attr(to_url)}">{esc(title[lang])}</a></html>\n')
    if untitled: print('docs: no Simplified Chinese title in web/docs-titles.txt for: ' + '; '.join(sorted(untitled)))
    # The raw files at the root (A.1): /llms.txt, /AGENTS.md, /registry.json.
    for rel in ['llms.txt', 'AGENTS.md', 'registry.json']:
        (ctx['out'] / rel).write_bytes((fw / rel).read_bytes())
    return {'pages': written + index, 'redirects': redirects}


def stage_routes(ctx):
    """The canvas routes the docs pages run as live stages: each component's specimen and each Lab
    experiment's page (for the build-time stills)."""
    canvas = {p['path'] for p in ctx['data']['pages']}
    reg = json.loads((ctx['fw'] / 'registry.json').read_text())
    out = [f'components/{i["name"].replace("_", "-")}/' for i in reg['items'] if i['name'] != 'base']
    out += [f'lab/{f.stem[4:]}/' for f in sorted((ctx['root'] / 'docs/site/en').glob('lab-*.md'))]
    return [r for r in out if r in canvas]


# The stage (I.2 item 7): the route's stage alone (`<path>#stage`), 280 px tall (200 on a phone), its
# specimen centred. Its stills are that route drawn inside the frame of the desktop box and of a 390 px
# phone's (STILLS, at scale 2, by web/build.py), each shown at its own size and centred, so a box of
# another width crops it where the live canvas would lay out the same specimen.
STILLS = {'wide': (718, 278), 'narrow': (358, 198)}


def stage_html(ctx, p, t, href):
    route = p.stage if p.stage is not None else p.canvas
    still = ctx['stills'].get(route, {})
    img = ''
    if 'wide' in still:
        w, h = STILLS['wide']
        img = f'<img src="{attr(ctx["base"] + still["wide"])}" alt="" width="{w}" height="{h}" decoding="async">'
        if 'narrow' in still: img = f'<picture><source media="(max-width: 719px)" srcset="{attr(ctx["base"] + still["narrow"])}" width="{STILLS["narrow"][0]}" height="{STILLS["narrow"][1]}">{img}</picture>'
    return (f'<figure class="stage" data-route="{attr((route or "") + "#stage")}"><div class="box"><button type="button" class="run" aria-label="{attr(t("web.docs.run"))}">{img}</button></div>'
            f'<figcaption><span class="state" data-idle="{attr(t("web.docs.stage"))}">{esc(t("web.docs.stage-still"))}</span><span class="acts"><a href="{href(route)}">{esc(t("web.docs.open"))}&nbsp;↗</a></span></figcaption></figure>')


def norm(p):
    parts = []
    for x in str(p).split('/'):
        if x in ('', '.'): continue
        if x == '..':
            if parts: parts.pop()
            continue
        parts.append(x)
    return '/'.join(parts)


def section_links(h, sections, prefix, here):
    """`DESIGN.md §1.8.2` (and `§1.15–§1.17` after it) in text, outside links and code, as links."""
    out, depth = [], {'a': 0, 'code': 0, 'pre': 0}
    for part in re.split(r'(<[^>]+>)', h):
        m = re.match(r'<(/?)(a|code|pre)\b', part)
        if m: depth[m.group(2)] += -1 if m.group(1) else 1
        if part.startswith('<') or any(depth.values()) or '§' not in part: out.append(part); continue
        def sub(m):
            num = m.group(2)
            hit = sections.get(num) or sections.get(num.rstrip('.'))
            while not hit and '.' in num:
                num = num.rsplit('.', 1)[0]; hit = sections.get(num)
            if not hit: return m.group(0)
            p, a = hit
            href = ('' if p is here else prefix + p.path) + ('#' + a if a else '')
            return f'{m.group(1)}<a href="{href or "#"}">§{m.group(2)}</a>'
        part = re.sub(r'((?:DESIGN\.md\s*)?)§(\d+(?:\.\d+)*)', sub, part)
        out.append(part)
    return ''.join(out)


def design_pages(fw, rel, langs, titled):
    """DESIGN.md cut at its sections: the introduction with the terms and §0; each `### 1.x` of the
    pipeline; each other `## n.` section whole. Headings move up so each page's own is its title. A
    section under MIN_WORDS joins a neighbour: one with subsections takes its first one in; a `## n.`
    section joins the introduction; any other the page before it, its headings one below that page's
    title."""
    kind, text = front((fw / rel).read_text())
    lines = text.split('\n')
    heads, fence = [], False
    for i, l in enumerate(lines):
        if l.startswith('```'): fence = not fence
        if fence: continue
        m = re.match(r'^(#{1,4}) (.*)$', l)
        if m: heads.append((i, len(m.group(1)), m.group(2)))
    cuts = [0]
    for i, lv, t in heads:
        num = re.match(r'^(\d+(?:\.\d+)*)\.?\s', t)
        if lv == 2 and num and num.group(1) != '0': cuts.append(i)
        elif lv == 3 and num and num.group(1).startswith('1.'): cuts.append(i)
    cuts.append(len(lines))
    head_at = {i: (lv, t) for i, lv, t in heads}
    def num_of(t):
        m = re.match(r'^(\d+(?:\.\d+)*)', t)
        return m.group(1) if m else None
    # Each page: its title's level and text, and its lines (headings rewritten where a section joined).
    chunks = []
    for a, b in zip(cuts, cuts[1:]):
        lv, t = head_at.get(a, (1, 'Design'))
        chunks.append({'level': lv if a else 1, 'title': t if a else 'Design', 'lines': lines[a:b]})
    def body_words(c): return words('\n'.join(c['lines'][1:]))
    def deepen(c, by):
        out, fence = [], False
        for l in c['lines']:
            if l.startswith('```'): fence = not fence
            out.append('#' * by + l if by > 0 and not fence and re.match(r'^#{1,5} ', l) else l)
        return out
    i = 1
    while i < len(chunks):
        c = chunks[i]
        if body_words(c) >= MIN_WORDS: i += 1; continue
        n, nxt = num_of(c['title']), chunks[i + 1] if i + 1 < len(chunks) else None
        if nxt and n and (num_of(nxt['title']) or '').startswith(n + '.'):
            c['lines'] = c['lines'] + nxt['lines']; del chunks[i + 1]; continue
        into = chunks[0] if c['level'] == 2 else chunks[i - 1]
        into['lines'] = into['lines'] + [''] + deepen(c, into['level'] + 1 - c['level'])
        del chunks[i]
    out, seen = [], set()
    for k, c in enumerate(chunks):
        title, level = c['title'], c['level']
        if k == 0:
            title_md, body = split_title('\n'.join(c['lines']))
            title, path, shift = title_md or 'Design', 'docs/concepts/design/', 0
        else:
            body = '\n'.join(c['lines'][1:])
            w = re.sub(r'^\d+(?:\.\d+)*\.?\s+', '', title)
            w = re.sub(r'\([^)]*\)', '', w)
            s = re.sub(r'[^a-z0-9]+', '-', w.lower()).strip('-')[:48].strip('-') or 'section'
            if s in seen: s = re.sub(r'[^0-9]+', '-', title.split()[0]).strip('-') + '-' + s
            seen.add(s)
            path, shift = f'docs/concepts/design/{s}/', -(level - 1)
        p = Page('design-' + (path.split('/')[-2] if k else 'index'), 'concepts', path, sub=('design', 'web.docs.design'), md={l: body for l in langs},
                 title=titled(title), content_lang={l: 'en' for l in langs}, src=('fw', rel))
        p.shift = shift
        # The section numbers this page holds: its own (no anchor) and its subsections' (by heading id,
        # as the page's renderer gives them).
        p.sections = []
        own = num_of(title) if k else None
        if own: p.sections.append((own, None))
        ids, fence = set(), False
        for l in body.split('\n'):
            if l.startswith('```'): fence = not fence
            if fence: continue
            m = re.match(r'^(#{1,6}) (.*)$', l)
            if not m: continue
            hid = slug(inline(m.group(2)), ids)
            n = num_of(m.group(2))
            if n: p.sections.append((n, hid))
        out.append(p)
    return out


def header_line(src, part):
    out, on = [], False
    for l in src.split('\n'):
        if not l.startswith('//!'): break
        t = l[3:].strip()
        if t.startswith(part): on = True; out.append(t[len(part):].strip()); continue
        if on and l.startswith('//!   '): out.append(t); continue
        on = False
    return ' '.join(out)


def examples_of(src):
    lines, out = src.split('\n'), []
    for i, l in enumerate(lines):
        if l.startswith('/// ') and i + 1 < len(lines) and lines[i + 1].startswith('pub fn ex_'):
            end = next((j for j in range(i + 1, len(lines)) if lines[j] == '}'), len(lines) - 1)
            # Each example is the call in a function named `view`, as an application writes it.
            body = re.sub(r'^pub fn ex_\d+\(\)', 'fn view()', '\n'.join(lines[i + 1:end + 1]))
            out.append((l[4:], body))
    return out


def pick(label, lang):
    z, sep, e = label.partition(' · ')
    return label if not sep else (e if lang == 'en' else z)


def closure(name, by_name):
    """The files `amenui add <name>` copies: the component's and everything it depends on."""
    seen, files, todo = set(), [], [name]
    while todo:
        n = todo.pop(0)
        if n in seen or n not in by_name: continue
        seen.add(n)
        for f in by_name[n].get('files', []):
            if f not in files: files.append(f)
        todo += by_name[n].get('dependencies', [])
    return files


def component_html(p, lang, tr, by_name, fw, link):
    """A component's page in the fixed order of D.5: (description), the stage, installation, usage,
    examples, API, keyboard and accessibility."""
    t = lambda k: tr(lang, k)
    name = p.keywords.split()[0]
    it = by_name[name]
    src = (fw / it['files'][0]).read_text() if it.get('files') else ''
    ex_file = fw / 'examples/components' / ({'scroll-area': 'scroll', 'scroll_area': 'scroll'}.get(name, name) + '.rs')
    exs = examples_of(ex_file.read_text()) if ex_file.exists() else []
    desc = it.get('description_en') if lang == 'en' and it.get('description_en') else it.get('description', '')
    ids, heads, parts = set(), [], []
    il = lambda s: inline(s, link)
    def h2(title):
        hid = slug(esc(title), ids); heads.append((2, esc(title), hid))
        return f'<h2 id="{hid}"><a href="#{hid}">{esc(title)}</a></h2>'
    # The registry's line: its first sentence is the lead (as on the index cards and in search), the
    # rest a paragraph under it.
    lead, rest = first_sentence(desc)
    parts.append(f'<p class="lead">{il(lead)}</p>' + (f'<p>{il(rest)}</p>' if rest else ''))
    parts.append('@@STAGE@@')
    parts.append(h2(t('docs.sec.install')))
    parts.append(f'<p>{il(t("docs.install.lead"))}</p>')
    parts.append(code_block(f'cargo run --release --bin amenui -- add {name} --to ../my-app', 'sh', t))
    parts.append(f'<p>{esc(t("docs.install.files"))}: ' + ', '.join(f'<code>{esc(f)}</code>' for f in closure(name, by_name)) + '</p>')
    parts.append(h2(t('docs.sec.usage')))
    fns = it.get('functions', [])
    # Usage: the import and the first example's call; an example that prints the same is not repeated.
    usage = exs[0][1] if exs else ''
    parts.append(code_block(f'use amenui::widgets::{{{", ".join(fns)}}};' + ('\n\n' + usage if usage else ''), 'rust', t))
    more = [(label, c) for label, c in exs if c != usage]
    if more:
        parts.append(h2(t('docs.sec.examples')))
        for ex_label, c in more:
            hid = slug(esc(pick(ex_label, lang)), ids)
            parts.append(f'<h3 id="{hid}"><a href="#{hid}">{esc(pick(ex_label, lang))}</a></h3>')
            parts.append(code_block(c, 'rust', t))
    parts.append(h2(t('docs.sec.api')))
    rows = [(t('docs.api.functions'), ', '.join(f'<code>{esc(f)}</code>' for f in fns)), (t('docs.api.props'), il(header_line(src, 'Props:'))),
            (t('docs.api.files'), ', '.join(f'<code>{esc(f)}</code>' for f in it.get('files', []))), (t('docs.api.deps'), esc(', '.join(it.get('dependencies', [])))),
            (t('docs.api.response'), il(it.get('response', ''))), (t('docs.api.recompose'), il(it.get('recompose', '')))]
    parts.append(kv_table(rows))
    parts.append(h2(t('docs.sec.keyboard')))
    parts.append(kv_table([(t('docs.kb.states'), il(header_line(src, 'States:'))), (t('docs.kb.keys'), il(header_line(src, 'Messages:')))]))
    parts.append(f'<p>{il(t("docs.kb.mirror"))}</p>')
    if it.get('recipe'):
        parts.append(f'<p class="also">{esc(t("docs.recipes"))}: <a href="@@recipe:{attr(it["recipe"])}@@">@@recipe-title:{attr(it["recipe"])}@@</a></p>')
    return '\n'.join(parts), heads


def kv_table(rows):
    body = ''.join(f'<tr><th scope="row">{esc(k)}</th><td>{v}</td></tr>' for k, v in rows if v)
    return f'<div class="table"><table class="kv"><tbody>{body}</tbody></table></div>'


def registry_md(lang, tr, items, ctx):
    t = lambda k: tr(lang, k)
    md = f'{t("docs.registry.lead")}\n\n| {t("docs.api.name")} | {t("docs.api.functions")} | {t("docs.api.files")} |\n|---|---|---|\n'
    for it in items:
        h = it['name'].replace('_', '-')
        md += f'| [{it["name"]}]({ctx["base"]}{"" if lang == "en" else lang + "/"}docs/components/{h}/) | `{", ".join(it.get("functions", []))}` | `{", ".join(it.get("files", []))}` |\n'
    md += f'\n[registry.json]({ctx["base"]}registry.json)\n'
    return md


# The page around the text (D.5).
def write_page(ctx, p, lang, nav, tr):
    t = lambda k: tr(lang, k)
    BASE, langs = ctx['base'], ctx['langs']
    prefix = '' if lang == 'en' else lang + '/'
    href = lambda path: BASE + prefix + path
    body, heads, _ = p.out[lang]
    # Links to a component's recipe page.
    def recipe(m):
        rel = m.group(1)
        q = next((x for x in nav['visible'] if x.src == ('fw', rel)), None)
        return href(q.path) if q else f'{GITHUB["fw"]}/blob/main/{rel}'
    body = re.sub(r'@@recipe:([^@]*)@@', recipe, body)
    def recipe_title(m):
        q = next((x for x in nav['visible'] if x.src == ('fw', m.group(1))), None)
        return esc(q.title[lang]) if q else esc(m.group(1).rsplit('/', 1)[-1])
    body = re.sub(r'@@recipe-title:([^@]*)@@', recipe_title, body)
    body = one_face(body)
    stage = ''
    if p.stage or p.canvas is not None and p.hidden:
        stage = stage_html(ctx, p, t, href)
    body = body.replace('@@STAGE@@', stage) if '@@STAGE@@' in body else (stage + body if stage and p.hidden else body)
    group = dict(nav['groups']).get(p.group)
    gname = t(group) if group else t('docs.recipes')
    # A recipe of the Lab or of a scene is under that section; every other page under its group.
    if p.hidden:
        sec = 'web.works' if (p.canvas or '').startswith('works/') else 'web.lab'
        gname = t(sec)
        crumbs = [f'<a href="{href("docs/")}">{esc(t("web.docs"))}</a>', f'<a href="{href(sec[4:] + "/")}">{esc(gname)}</a>']
    else:
        crumbs = [f'<a href="{href("docs/")}">{esc(t("web.docs"))}</a>', f'<a href="{href("docs/")}#{p.group}">{esc(gname.lower() if lang == "en" else gname)}</a>']
    if p.sub: crumbs.append(esc(sub_name(p.sub, t).lower() if lang == 'en' else sub_name(p.sub, t)))
    path_line = ' / '.join(crumbs) + ' /'
    # Below 1,080 px the path line opens the sidebar: the same path as text, with the page's name.
    names = [t('web.docs'), gname] + ([sub_name(p.sub, t)] if p.sub else []) + [label(p.title[lang])]
    path_open = ' / '.join(label_html(n.lower() if lang == 'en' else n) for n in names)
    title = p.title[lang]
    toc = [(txt, hid) for lv, txt, hid in heads if lv == 2]
    cl = p.content_lang[lang]
    only_en = cl != lang
    vis = nav['visible']
    i = vis.index(p) if p in vis else -1
    prev = vis[i - 1] if i > 0 else None
    nxt = vis[i + 1] if 0 <= i < len(vis) - 1 else None
    pn = ''
    if prev or nxt:
        cell = lambda q, cls: f'<a class="{cls}" href="{href(q.path)}" rel="{cls}"><small>{"← " if cls == "prev" else ""}{q.num}{" →" if cls == "next" else ""}</small><span>{label_html(q.title[lang])}</span></a>' if q else '<span></span>'
        pn = f'<nav class="pn" aria-label="{attr(t("web.docs.pages"))}">' + cell(prev, 'prev') + cell(nxt, 'next') + '</nav>'
    edit = raw = ''
    if p.src:
        repo, rel = p.src
        rel = rel.replace('{lang}', cl)
        verb = 'edit' if rel.endswith('.md') else 'blob'
        edit = f'<a href="{GITHUB[repo]}/{verb}/main/{rel}" rel="noopener">{esc(t("web.docs.edit") if verb == "edit" else t("web.docs.source"))}&nbsp;↗</a>'
    if p.group != 'components' and p.key != 'registry':
        raw = f'<a href="index.md">{esc(t("web.docs.raw"))}&nbsp;↗</a>'
        (ctx['out'] / prefix / p.path).mkdir(parents=True, exist_ok=True)
        (ctx['out'] / prefix / p.path / 'index.md').write_text(f'# {title}\n' + p.md[lang])
    toc_list = ''.join(f'<li><a href="#{hid}">{txt}</a></li>' for txt, hid in toc)
    toc_html = f'<aside class="toc" aria-label="{attr(t("docs.on-this-page"))}"><div class="toc-in">' + (f'<p class="label">{esc(t("docs.on-this-page"))}</p><ol>{toc_list}</ol>' if toc else '') + f'<p class="links">{edit}{raw}</p></div></aside>'
    toc_inline = f'<details class="toc-inline"><summary>{esc(t("docs.on-this-page"))} ▾</summary><ol>{toc_list}</ol></details>' if toc else ''
    note = f'<p class="only-en">{esc(t("web.docs.english"))}</p>' if only_en else ''
    num = f'<span class="num">{p.num}</span>' if p.num else ''
    main = (f'<main id="main" class="doc-main"><p class="path"><a class="path-open" href="#side" aria-controls="side">{path_open} <span aria-hidden="true">▾</span></a><span class="path-text">{path_line}</span></p>'
            f'<header class="title"><h1{" class=" + chr(34) + "cjk" + chr(34) if has_cjk(title) else ""}>{inline(title)}</h1>{num}</header>{note}{toc_inline}'
            f'<article class="prose"{" lang=" + chr(34) + cl + chr(34) if cl != lang else ""}>{body}</article>{pn}</main>')
    side = sidebar(nav, p, lang, t, href)
    description = desc_of(body) or t('web.description')
    page = ctx['frame'](lang=lang, path=p.path, title=f'{label(title)} · amenui', description=description, main=f'<div class="docs">{side}{main}{toc_html}</div>')
    d = ctx['out'] / prefix / p.path
    d.mkdir(parents=True, exist_ok=True)
    (d / 'index.html').write_text(page)
    crumb = gname + (' · ' + sub_name(p.sub, t) if p.sub else '')
    return {'lang': lang, 'url': p.path, 'title': label(title), 'crumb': crumb, 'body': body, 'heads': heads, 'keywords': p.keywords, 'canvas': p.canvas, 'size': len(page.encode())}


def one_face(h):
    """A heading that holds Chinese is set in the system's CJK sans whole, Latin included (I.3 ruling
    11): it gets the class `cjk`."""
    return re.sub(r'<(h[1-4])((?: id="[^"]*")?)>(.*?)</\1>', lambda m: f'<{m.group(1)} class="cjk"{m.group(2)}>{m.group(3)}</{m.group(1)}>' if has_cjk(plain(m.group(3))) else m.group(0), h, flags=re.S)


def sub_name(sub, t):
    return t(sub[1])


def desc_of(body):
    m = re.search(r'<p[^>]*>(.*?)</p>', body, re.S)
    if not m: return ''
    s = plain(m.group(1))
    return s if len(s) <= 180 else s[:177].rsplit(' ', 1)[0] + '…'


def sidebar(nav, here, lang, t, href):
    """The sidebar (D.5): the six groups, numbered; their pages; sub-groups closed but the current."""
    out = [f'<nav class="side" id="side" aria-label="{attr(t("web.docs"))}"><div class="side-in"><a class="side-close" href="#" aria-label="{attr(t("web.close"))}">×</a>']
    for gi, (g, key) in enumerate(nav['groups']):
        items = [p for p in nav['visible'] if p.group == g]
        out.append(f'<p class="g"><small>{gi + 1:02d}</small>{esc(t(key))}</p><ul>')
        subs, done = {}, set()
        for p in items:
            if p.sub: subs.setdefault(p.sub[0], []).append(p)
        for p in items:
            if p.sub:
                sid = p.sub[0]
                if sid in done: continue
                done.add(sid)
                group = subs[sid]
                cur = here in group
                out.append(f'<li class="sub"><details{" open" if cur else ""}><summary>{esc(sub_name(p.sub, t))} <small>{len(group)}</small></summary><ul>')
                for q in group: out.append(row(q, here, lang, href))
                out.append('</ul></details></li>')
            else:
                out.append(row(p, here, lang, href))
        out.append('</ul>')
    out.append('</div></nav>')
    return ''.join(out)


def row(p, here, lang, href):
    cur = p is here
    return f'<li><a href="{href(p.path)}"' + (' aria-current="page"><i class="here" aria-hidden="true">▸</i>' if cur else '>') + f'{label_html(p.title[lang])}</a></li>'


def write_index(ctx, nav, tr):
    """The docs index (/docs/): every group with its section header (D.4) and its pages."""
    written = []
    for lang in ctx['langs']:
        t = lambda k: tr(lang, k)
        prefix = '' if lang == 'en' else lang + '/'
        href = lambda path: ctx['base'] + prefix + path
        parts = [f'<p class="lead">{inline(t("web.docs.lead"))}</p>']
        for gi, (g, key) in enumerate(nav['groups']):
            items = [p for p in nav['visible'] if p.group == g]
            parts.append(f'<h2 class="sec" id="{g}"><small>{gi + 1:02d}</small>{esc(t(key))}</h2><ul class="idx' + (' compact' if g == 'components' else '') + '">')
            last = None
            for p in items:
                if p.sub and p.sub != last and g != 'components':
                    parts.append(f'</ul><h3>{esc(sub_name(p.sub, t))}</h3><ul class="idx compact">')
                if p.sub and p.sub != last and g == 'components':
                    parts.append(f'<li class="subhead">{esc(sub_name(p.sub, t))}</li>')
                last = p.sub
                d = desc_of(p.out[lang][0]) if g not in ('components',) and not (p.sub and g != 'components') else ''
                parts.append(f'<li><a href="{href(p.path)}"><small>{p.num}</small><span>{label_html(p.title[lang])}</span></a>' + (f'<p>{esc(d)}</p>' if d else '') + '</li>')
            parts.append('</ul>')
        body = '\n'.join(parts)
        main = (f'<main id="main" class="doc-main index"><p class="path"><span class="path-text">{esc(t("web.docs"))} /</span><a class="path-open" href="#side" aria-controls="side">{esc(t("web.docs"))} / <span aria-hidden="true">▾</span></a></p>'
                + one_face(f'<header class="title"><h1>{esc(t("web.docs.title"))}</h1></header>') + f'<article class="prose">{body}</article></main>')
        here = None
        side = sidebar(nav, here, lang, t, href)
        page = ctx['frame'](lang=lang, path='docs/', title=f'{t("web.docs.title")} · amenui', description=t('web.docs.lead'), main=f'<div class="docs">{side}{main}<aside class="toc"></aside></div>')
        d = ctx['out'] / prefix / 'docs'
        d.mkdir(parents=True, exist_ok=True)
        (d / 'index.html').write_text(page)
        written.append({'lang': lang, 'url': 'docs/', 'title': t('web.docs.title'), 'crumb': t('web.docs'), 'body': '', 'heads': [], 'keywords': '', 'size': len(page.encode())})
    return written


def not_found(ctx, frame, search):
    """The 404 page (I.2 item 11): the bar, "Not found" at 44 px, one line and three doors, and no
    module. One file serves every missing address, so it holds each language's bar and text and
    shows the one its address is in. Older addresses (`#button`) still lead to their pages."""
    BASE, langs = ctx['base'], ctx['langs']
    def tr(lang, k): return ctx['cats'][lang].get(k) or ctx['cats']['en'][k]
    redirect = '<script>(()=>{const m=%s,h=decodeURIComponent(location.hash.slice(1));if(h&&!h.startsWith("look:")&&m[h]!==undefined)location.replace(%s+m[h]);})()</script>' % (json.dumps(ctx['redirects'], separators=(',', ':')), json.dumps(BASE))
    parts, cfgs = {}, {}
    for lang in langs:
        t = lambda k: tr(lang, k)
        href = lambda p: BASE + ('' if lang == 'en' else lang + '/') + p
        doors = ''.join(f'<a class="btn" href="{href(p)}">{esc(t(k))}</a>' for p, k in [('', 'web.not-found.home'), ('docs/', 'web.docs'), ('components/', 'web.components')])
        main = f'<main id="main" class="nf"><h1{" class=" + chr(34) + "cjk" + chr(34) if has_cjk(t("web.not-found")) else ""}>{esc(t("web.not-found"))}</h1><p>{esc(t("web.not-found-text"))}</p><p class="doors">{doors}</p></main>'
        page = frame(lang=lang, path='', title=f'{t("web.not-found")} · amenui', description=t('web.description'), main=main).replace(' aria-current="page"', '')
        parts[lang] = re.search(r'<body>\n(.*?)\n<footer', page, re.S).group(1)
        cfgs[lang] = json.loads(re.search(r'window\.SEARCH=(\{.*?\});\(', search(lang)).group(1))
        if lang == 'en': head, tail = page.split('<body>\n', 1)[0], page.split('</main>\n', 1)[1]
    others = ''.join(f'<template data-lang="{l}">{parts[l]}</template>' for l in langs if l != 'en')
    # Before the search's script binds the bar's button: the language's bar and text; after it, the
    # language's search.
    pick = ('<script>(()=>{const B=%s;for(const t of document.querySelectorAll("template[data-lang]")){const l=t.dataset.lang;if(!location.pathname.startsWith(B+l+"/"))continue;'
            'document.getElementById("nf").replaceWith(t.content);document.documentElement.lang=l;document.title=document.querySelector(".nf h1").textContent+" · amenui";window.NF=l}})()</script>') % json.dumps(BASE)
    then = '<script>if(window.NF)Object.assign(window.SEARCH,%s[window.NF])</script>' % json.dumps({l: cfgs[l] for l in langs if l != 'en'}, ensure_ascii=False, separators=(',', ':'))
    head = head.replace('<head>\n', '<head>\n' + redirect + '\n', 1)
    tail = tail.replace('@@SEARCH-INDEX@@', pick + search('en') + then + '\n')
    return f'{head}<body>\n<div id="nf">{parts["en"]}</div>{others}\n{tail}'
