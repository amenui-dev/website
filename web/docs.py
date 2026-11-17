# The docs as plain HTML pages (site design A.4, A.6, D.5): one page per document, in English at the
# root and in Simplified Chinese under zh-Hans/, written at build time from
#
#   docs/site/{en,zh-Hans}/*.md          the site's own pages (start, guides, concepts, reference, and
#                                        the Lab's and the body scene's recipes)
#   amenui: docs/**/*.md                 grouped by their front matter's kind; DESIGN.md is cut into
#                                        one page per section
#   amenui: registry.json, widgets,      one page per component: title, description, the live stage,
#           examples/components/*.rs     installation, usage, examples, API, keyboard
#   amenui: AGENTS.md, llms.txt,         reference and release notes; also served as they are at the
#           CHANGELOG.md, registry.json  site's root
#
# `build(ctx)` writes the pages and returns what the search index needs (`web/build.py` calls it).
import html, json, pathlib, re
from md import Doc, attr, code_block, esc, inline

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
    return re.sub(r'\s+', ' ', html.unescape(re.sub(r'<[^>]+>', '', h))).strip()


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
        if not (name.startswith('lab-') or name.startswith('scene-')): continue
        if any(name in v for v in SITE.values()): continue
        short = name.split('-', 1)[1]
        md, title, cl = {}, {}, {}
        for lang in langs:
            ff = root / 'docs/site' / lang / f'{name}.md'
            c = lang if ff.exists() else 'en'
            if not ff.exists(): ff = f
            t, body = split_title(front(ff.read_text())[1])
            md[lang], title[lang], cl[lang] = body, t or short, c
        canvas = f'lab/{short}/' if name.startswith('lab-') else f'scenes/{short}/'
        if canvas not in canvas_paths: canvas = '' if short == 'body' else None
        pages.append(Page(f'recipe-{short}', 'recipes', f'docs/recipes/{short}/', md=md, title=title, content_lang=cl, src=('site', f'docs/site/{{lang}}/{name}.md'), hidden=True, canvas=canvas))
    placed = {s for v in SITE.values() for s in v}
    for f in (root / 'docs/site/en').glob('*.md'):
        assert f.stem in placed or f.stem.startswith(('lab-', 'scene-')), f'docs/site/en/{f.name} has no place in the docs tree (web/docs.py SITE)'

    # amenui's docs, by kind.
    def fw_page(rel, group, path, key, sub=None, title=None):
        kind, text = front((fw / rel).read_text())
        t, body = split_title(text)
        return Page(key, group, path, sub=sub, md={l: body for l in langs}, title={l: title or t or key for l in langs}, content_lang={l: 'en' for l in langs}, src=('fw', rel))
    fwdocs = sorted(p.relative_to(fw).as_posix() for p in (fw / 'docs').rglob('*.md'))
    kinds = {r: front((fw / r).read_text())[0] for r in fwdocs}
    for r, k in kinds.items(): assert k in ('readme', 'howto', 'reference', 'explanation'), f'amenui {r}: front matter kind {k!r}'
    for r in fwdocs:
        if kinds[r] == 'readme': pages.append(fw_page(r, 'start', 'docs/start/documentation/', 'documentation'))
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
        if kinds[r] == 'explanation': design += design_pages(fw, r, langs)
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
    redirects = dict(ctx['data']['redirects'])
    for p in allp:
        if p.hash: redirects[p.hash] = p.path
    # Section numbers of DESIGN.md (§1.8.2) to their pages and anchors.
    sections = {}
    for p in design:
        for num, anchor in p.sections: sections[num] = (p, anchor)

    def resolver(p, lang):
        prefix = BASE + ('' if lang == 'en' else lang + '/')
        def link(url):
            gh = re.match(r'^https://github\.com/amenui-dev/(website|amenui)/(?:blob|tree)/main/([^#]+)(#.*)?$', url)
            if gh:
                q = files.get(('site' if gh.group(1) == 'website' else 'fw', re.sub(r'^docs/site/[^/]+/', 'docs/site/' + lang + '/', gh.group(2)) if gh.group(1) == 'website' else gh.group(2)))
                if q: return (prefix + q.path + (gh.group(3) or ''), False)
            if re.match(r'^[a-z]+:', url):
                site = re.match(r'^https://amenui-dev\.github\.io/website/(.*)$', url)
                return (prefix + site.group(1), False) if site else (url, True)
            if url.startswith('/'): return (url, False)
            if url.startswith('#'):
                h = url[1:]
                if h in redirects: return (prefix + redirects[h], False)
                return ('@@#' + h + '@@', False)
            path, _, frag = url.partition('#')
            repo, rel = p.src if p.src else ('site', '')
            rel = rel.replace('{lang}', lang)
            base = pathlib.PurePosixPath(rel).parent
            m = re.match(r'^md/[^/]+/(.+)\.md$', path)
            if m and m.group(1) in redirects: return (prefix + redirects[m.group(1)], False)
            target = norm(base / path)
            q = files.get((repo, target)) or files.get((repo, target.rstrip('/')))
            if q: return (prefix + q.path + ('#' + frag if frag else ''), False)
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
    # The raw files at the root (A.1): /llms.txt, /AGENTS.md, /registry.json.
    for rel in ['llms.txt', 'AGENTS.md', 'registry.json']:
        (ctx['out'] / rel).write_bytes((fw / rel).read_bytes())
    return {'pages': written + index, 'redirects': redirects}


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


def design_pages(fw, rel, langs):
    """DESIGN.md cut at its sections: the introduction with the terms and §0; each `### 1.x` of the
    pipeline; each other `## n.` section whole. Headings move up so each page's own is its title."""
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
    out, seen = [], set()
    for a, b in zip(cuts, cuts[1:]):
        chunk = lines[a:b]
        first = next((h for h in heads if h[0] == a), None)
        if a == 0:
            title, level, slug_, path = 'Design', 1, '', 'docs/concepts/design/'
            body = '\n'.join(chunk)
            title_md, body = split_title(body)
            title = title_md or 'Design'
            shift = 0
        else:
            level, title = first[1], first[2]
            body = '\n'.join(chunk[1:])
            words = re.sub(r'^\d+(?:\.\d+)*\.?\s+', '', title)
            words = re.sub(r'\([^)]*\)', '', words)
            s = re.sub(r'[^a-z0-9]+', '-', words.lower()).strip('-')[:48].strip('-') or 'section'
            if s in seen: s = re.sub(r'[^0-9]+', '-', title.split()[0]).strip('-') + '-' + s
            seen.add(s)
            path = f'docs/concepts/design/{s}/'
            shift = -(level - 1)
        p = Page('design-' + (path.split('/')[-2] if a else 'index'), 'concepts', path, sub=('design', 'web.docs.design'), md={l: body for l in langs},
                 title={l: title for l in langs}, content_lang={l: 'en' for l in langs}, src=('fw', rel))
        p.shift = shift
        # The section numbers this page holds: its own (no anchor) and its subsections' (by heading id).
        p.sections = []
        own = re.match(r'^(\d+(?:\.\d+)*)', title)
        if own: p.sections.append((own.group(1), None))
        from md import slug
        ids = set()
        fence = False
        for l in chunk[1:] if a else chunk:
            if l.startswith('```'): fence = not fence
            if fence: continue
            m = re.match(r'^(#{2,6}) (.*)$', l)
            if not m: continue
            hid = slug(inline(m.group(2)), ids)
            n = re.match(r'^(\d+(?:\.\d+)*)', m.group(2))
            if n: p.sections.append((n.group(1), hid))
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
            out.append((l[4:], '\n'.join(lines[i + 1:end + 1])))
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
        from md import slug
        hid = slug(esc(title), ids); heads.append((2, esc(title), hid))
        return f'<h2 id="{hid}"><a href="#{hid}">{esc(title)}</a></h2>'
    parts.append(f'<p class="lead">{il(desc)}</p>')
    parts.append('@@STAGE@@')
    parts.append(h2(t('docs.sec.install')))
    parts.append(f'<p>{il(t("docs.install.lead"))}</p>')
    parts.append(code_block(f'cargo run --release --bin amenui -- add {name} --to ../my-app', 'sh', t))
    parts.append(f'<p>{esc(t("docs.install.files"))}: ' + ', '.join(f'<code>{esc(f)}</code>' for f in closure(name, by_name)) + '</p>')
    parts.append(h2(t('docs.sec.usage')))
    fns = it.get('functions', [])
    parts.append(code_block(f'use amenui::widgets::{{{", ".join(fns)}}};' + ('\n\n' + exs[0][1] if exs else ''), 'rust', t))
    if exs:
        parts.append(h2(t('docs.sec.examples')))
        for label, c in exs:
            from md import slug
            hid = slug(esc(pick(label, lang)), ids)
            parts.append(f'<h3 id="{hid}"><a href="#{hid}">{esc(pick(label, lang))}</a></h3>')
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
    stage = ''
    if p.stage or p.canvas is not None and p.hidden:
        stage = ctx['stage_html'](p, lang, href)
    body = body.replace('@@STAGE@@', stage) if '@@STAGE@@' in body else (stage + body if stage and p.hidden else body)
    group = dict(nav['groups']).get(p.group)
    gname = t(group) if group else t('docs.recipes')
    # A recipe of the Lab or of a scene is under that section; every other page under its group.
    if p.hidden:
        sec = 'web.scenes' if p.key == 'recipe-body' else 'web.lab'
        gname = t(sec)
        crumbs = [f'<a href="{href("docs/")}">{esc(t("web.docs"))}</a>', f'<a href="{href(sec[4:] + "/")}">{esc(gname)}</a>']
    else:
        crumbs = [f'<a href="{href("docs/")}">{esc(t("web.docs"))}</a>', f'<a href="{href("docs/")}#{p.group}">{esc(gname.lower() if lang == "en" else gname)}</a>']
    if p.sub: crumbs.append(esc(sub_name(p.sub, t).lower() if lang == 'en' else sub_name(p.sub, t)))
    path_line = ' / '.join(crumbs) + ' /'
    # Below 1,080 px the path line opens the sidebar: the same path as text, with the page's name.
    names = [t('web.docs'), gname] + ([sub_name(p.sub, t)] if p.sub else []) + [p.title[lang]]
    path_open = esc(' / '.join(n.lower() if lang == 'en' else n for n in names))
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
        cell = lambda q, cls: f'<a class="{cls}" href="{href(q.path)}" rel="{cls}"><small>{"← " if cls == "prev" else ""}{q.num}{" →" if cls == "next" else ""}</small><span>{esc(q.title[lang])}</span></a>' if q else '<span></span>'
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
    main = (f'<main id="content" class="doc-main"><p class="path"><a class="path-open" href="#side" aria-controls="side">{path_open} <span aria-hidden="true">▾</span></a><span class="path-text">{path_line}</span></p>'
            f'<header class="title"><h1>{esc(title)}</h1>{num}</header>{note}{toc_inline}'
            f'<article class="prose"{" lang=" + chr(34) + cl + chr(34) if cl != lang else ""}>{body}</article>{pn}</main>')
    side = sidebar(nav, p, lang, t, href)
    description = desc_of(body) or t('web.description')
    page = ctx['frame'](lang=lang, path=p.path, title=f'{title} · amenui', description=description, main=f'<div class="docs">{side}{main}{toc_html}</div>')
    d = ctx['out'] / prefix / p.path
    d.mkdir(parents=True, exist_ok=True)
    (d / 'index.html').write_text(page)
    crumb = gname + (' · ' + sub_name(p.sub, t) if p.sub else '')
    return {'lang': lang, 'url': p.path, 'title': title, 'crumb': crumb, 'body': body, 'heads': heads, 'keywords': p.keywords, 'size': len(page.encode())}


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
    return f'<li><a href="{href(p.path)}"' + (' aria-current="page"><i class="here" aria-hidden="true">▸</i>' if cur else '>') + f'{esc(p.title[lang])}</a></li>'


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
                parts.append(f'<li><a href="{href(p.path)}"><small>{p.num}</small><span>{esc(p.title[lang])}</span></a>' + (f'<p>{esc(d)}</p>' if d else '') + '</li>')
            parts.append('</ul>')
        body = '\n'.join(parts)
        main = (f'<main id="content" class="doc-main index"><p class="path"><span class="path-text">{esc(t("web.docs"))} /</span><a class="path-open" href="#side" aria-controls="side">{esc(t("web.docs"))} / <span aria-hidden="true">▾</span></a></p>'
                f'<header class="title"><h1>{esc(t("web.docs.title"))}</h1></header><article class="prose">{body}</article></main>')
        here = None
        side = sidebar(nav, here, lang, t, href)
        page = ctx['frame'](lang=lang, path='docs/', title=f'{t("web.docs.title")} · amenui', description=t('web.docs.lead'), main=f'<div class="docs">{side}{main}<aside class="toc"></aside></div>')
        d = ctx['out'] / prefix / 'docs'
        d.mkdir(parents=True, exist_ok=True)
        (d / 'index.html').write_text(page)
        written.append({'lang': lang, 'url': 'docs/', 'title': t('web.docs.title'), 'crumb': t('web.docs'), 'body': '', 'heads': [], 'keywords': '', 'size': len(page.encode())})
    return written
