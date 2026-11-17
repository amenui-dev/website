# Markdown to HTML for the docs pages (web/docs.py): the subset the docs are written in. Blocks:
# headings, paragraphs, fenced code, tables, lists (nested by indentation), block quotes (as notes) and
# rules. Inline: code spans, links, images, `**strong**` and `*emphasis*`. Raw HTML in the text is
# escaped, never passed through. Links go through a function the caller gives, so every target can be
# rewritten to the site's own paths.
import html, re

def esc(s): return html.escape(s, quote=False)
def attr(s): return html.escape(s, quote=True)

def slug(text, seen):
    """A heading's id: lowercase words joined by `-` (CJK kept), unique within the page."""
    t = re.sub(r'<[^>]+>', '', text)
    t = re.sub(r'[`*_]', '', html.unescape(t)).lower()
    t = re.sub(r'[^\w一-鿿㐀-䶿]+', '-', t).strip('-') or 'section'
    base, n = t, 2
    while t in seen: t = f'{base}-{n}'; n += 1
    seen.add(t)
    return t

def cjk(c): return '⺀' <= c <= '鿿' or '豈' <= c <= '﫿' or '＀' <= c <= '￯' or '　' <= c <= '〿'

def join_lines(lines):
    """A paragraph's lines as one run: a space between lines, none between two CJK characters."""
    out = ''
    for l in lines:
        l = l.strip()
        if not out: out = l; continue
        out += ('' if cjk(out[-1]) and l and cjk(l[0]) else ' ') + l
    return out

# Inline: code spans are cut out first so nothing inside them is read as markup.
CODE = re.compile(r'(`+)(.+?)\1')
LINK = re.compile(r'(!?)\[((?:[^\[\]]|\[[^\]]*\])+)\]\(([^)\s]+)(?:\s+"[^"]*")?\)')
AUTO = re.compile(r'<(https?://[^>\s]+)>')

def inline(s, link=lambda u: (u, False)):
    parts, last = [], 0
    for m in CODE.finditer(s):
        parts.append(('t', s[last:m.start()]))
        parts.append(('c', m.group(2).strip() if m.group(2).strip() else m.group(2)))
        last = m.end()
    parts.append(('t', s[last:]))
    out = []
    for kind, t in parts:
        if kind == 'c': out.append('<code>' + esc(t) + '</code>'); continue
        out.append(text_run(t, link))
    return ''.join(out)

def text_run(t, link):
    out, last = [], 0
    for m in LINK.finditer(t):
        out.append(emph(esc(t[last:m.start()]), link))
        img, label, url = m.groups()
        href, ext = link(url)
        if img: out.append(f'<img src="{attr(href)}" alt="{attr(label)}" loading="lazy">')
        else: out.append(f'<a href="{attr(href)}"' + (' rel="noopener"' if ext else '') + '>' + inline(label, link) + ('&nbsp;↗' if ext and not label.endswith('↗') else '') + '</a>')
        last = m.end()
    out.append(emph(esc(t[last:]), link))
    return ''.join(out)

def emph(s, link):
    s = AUTO.sub(lambda m: f'<a href="{attr(m.group(1))}" rel="noopener">{m.group(1)}&nbsp;↗</a>', s.replace('&lt;', '<').replace('&gt;', '>')) if '&lt;http' in s else s
    s = re.sub(r'\*\*(?=\S)(.+?)(?<=\S)\*\*', r'<strong>\1</strong>', s)
    s = re.sub(r'(?<![\w*])\*(?=[^\s*])(.+?)(?<=[^\s*])\*(?![\w*])', r'<em>\1</em>', s)
    s = re.sub(r'(?<![\w])_(?=\S)([^_]+?)(?<=\S)_(?![\w])', r'<em>\1</em>', s)
    return s

# Code: four colours (D.5): ink for code, dim for comments, second for strings and numbers.
def highlight(src, lang):
    lang = (lang or '').lower()
    hash_comments = lang in ('sh', 'bash', 'shell', 'toml', 'text', 'python', 'py', 'yaml', 'txt')
    if lang in ('text', 'txt', ''): return esc(src)
    tok = re.compile(r'(?P<c>//[^\n]*|/\*.*?\*/' + (r'|#[^\n]*' if hash_comments else '') + r')|(?P<s>"(?:[^"\\\n]|\\.)*"|\'(?:[^\'\\\n]|\\.)*\'(?!\w))|(?P<n>\b\d[\d_]*(?:\.\d+)?(?:e-?\d+)?(?:f32|f64|u8|u16|u32|u64|usize|i32|i64|px)?\b)', re.S)
    out, last = [], 0
    for m in tok.finditer(src):
        out.append(esc(src[last:m.start()]))
        cls = 'c' if m.group('c') else 's' if m.group('s') else 'n'
        # Lifetimes and char literals in Rust look like quotes: leave a lone `'a` alone.
        if cls == 's' and m.group(0).startswith("'") and lang == 'rust' and len(m.group(0)) > 4: out.append(esc(m.group(0)))
        else: out.append(f'<span class="{cls}">{esc(m.group(0))}</span>')
        last = m.end()
    out.append(esc(src[last:]))
    return ''.join(out)

def code_block(src, lang, t):
    """A code block: a 28 px header with the language and `copy`, the lines (numbered past six)."""
    lines = src.split('\n')
    body = highlight(src, lang)
    numbered = len(lines) > 6
    if numbered: body = '\n'.join(f'<span class="l">{l}</span>' for l in body.split('\n'))
    name = lang or 'text'
    return (f'<figure class="code"><figcaption><span>{esc(name)}</span><button type="button" class="copy" hidden>{esc(t("web.copy"))}</button></figcaption>'
            + ('<pre class="num">' if numbered else '<pre>') + f'<code>{body}</code></pre></figure>')

def table(rows, link):
    def cells(l):
        l = l.strip()
        if l.startswith('|'): l = l[1:]
        if l.endswith('|') and not l.endswith('\\|'): l = l[:-1]
        return [c.strip().replace('\\|', '|') for c in re.split(r'(?<!\\)\|', l)]
    head = cells(rows[0])
    body = [cells(r) for r in rows[2:]]
    align = []
    for c in cells(rows[1]): align.append('r' if c.endswith(':') and not c.startswith(':') else '')
    # Numbers right-aligned: a column whose every filled cell is a number (with a unit).
    numre = re.compile(r'^[−\-+]?[\d.,]+\s*(?:%|px|ms|s|kb|mb|×|x)?$', re.I)
    for j in range(len(head)):
        col = [r[j] for r in body if j < len(r) and r[j]]
        if col and all(numre.match(re.sub(r'`', '', c)) for c in col):
            while len(align) <= j: align.append('')
            align[j] = 'r'
    def cls(j): return ' class="r"' if j < len(align) and align[j] == 'r' else ''
    h = ''.join(f'<th{cls(j)}>{inline(c, link)}</th>' for j, c in enumerate(head))
    b = ''.join('<tr>' + ''.join(f'<td{cls(j)}>{inline(c, link)}</td>' for j, c in enumerate(r)) + '</tr>' for r in body)
    return f'<div class="table"><table><thead><tr>{h}</tr></thead><tbody>{b}</tbody></table></div>'

LIST = re.compile(r'^(\s*)([-*+]|\d+[.)])\s+(.*)$')
FENCE = re.compile(r'^(\s*)(```+|~~~+)\s*([\w+-]*)')

class Doc:
    """One page's blocks rendered: `html`, its headings [(level, text, id)], the first paragraph."""
    def __init__(self, md, link=lambda u: (u, False), t=lambda k: k, shift=0, seen=None):
        self.link, self.t, self.shift = link, t, shift
        self.headings, self.seen = [], seen if seen is not None else set()
        self.html = self.blocks(md.split('\n'))

    def blocks(self, lines, tight=False):
        out, i, n = [], 0, len(lines)
        while i < n:
            l = lines[i]
            if not l.strip(): i += 1; continue
            m = FENCE.match(l)
            if m:
                ind, fence, lang = m.group(1), m.group(2), m.group(3)
                j, body = i + 1, []
                while j < n and not lines[j].strip().startswith(fence[:3]):
                    body.append(lines[j][len(ind):] if lines[j].startswith(ind) else lines[j].lstrip()); j += 1
                out.append(code_block('\n'.join(body).rstrip('\n'), lang, self.t))
                i = j + 1; continue
            m = re.match(r'^(#{1,6})\s+(.*?)\s*#*$', l)
            if m:
                level = min(6, len(m.group(1)) + self.shift)
                content = inline(m.group(2), self.link)
                hid = slug(content, self.seen)
                self.headings.append((level, content, hid, m.group(2)))
                tag = f'h{min(level, 4)}'
                out.append(f'<{tag} id="{hid}"><a href="#{hid}">{content}</a></{tag}>' if level >= 2 else f'<h1 id="{hid}">{content}</h1>')
                i += 1; continue
            if re.match(r'^\s*([-*_])(\s*\1){2,}\s*$', l):
                out.append('<hr>'); i += 1; continue
            if l.lstrip().startswith('|') and i + 1 < n and re.match(r'^\s*\|?\s*:?-{2,}', lines[i + 1]):
                j = i
                while j < n and lines[j].strip().startswith('|'): j += 1
                out.append(table(lines[i:j], self.link)); i = j; continue
            if l.lstrip().startswith('>'):
                j, body = i, []
                while j < n and lines[j].lstrip().startswith('>'):
                    body.append(re.sub(r'^\s*>\s?', '', lines[j])); j += 1
                text = '\n'.join(body)
                warn = bool(re.match(r'^\s*(\*\*)?(warning|caution|!)', text, re.I))
                out.append(f'<aside class="note{" warn" if warn else ""}"><span class="mark" aria-hidden="true">{"!" if warn else "i"}</span><div>{self.blocks(body)}</div></aside>')
                i = j; continue
            m = LIST.match(l)
            if m:
                i = self.list(lines, i, out); continue
            j, para = i, []
            while j < n and lines[j].strip() and not FENCE.match(lines[j]) and not re.match(r'^#{1,6}\s', lines[j]) and not (lines[j].lstrip().startswith('|') and j + 1 < n and re.match(r'^\s*\|?\s*:?-{2,}', lines[j + 1])) and not (j > i and LIST.match(lines[j])) and not lines[j].lstrip().startswith('>'):
                para.append(lines[j]); j += 1
            text = inline(join_lines(para), self.link)
            out.append(text if tight else f'<p>{text}</p>')
            i = j
        return '\n'.join(out)

    def list(self, lines, i, out):
        m = LIST.match(lines[i])
        ind = len(m.group(1))
        ordered = m.group(2)[0].isdigit()
        start = int(re.match(r'\d+', m.group(2)).group(0)) if ordered else 1
        items, n, loose = [], len(lines), False
        while i < n:
            m = LIST.match(lines[i])
            if not m or len(m.group(1)) != ind or m.group(2)[0].isdigit() != ordered: break
            body = [m.group(3)]
            cont = len(m.group(1)) + len(m.group(2)) + 1
            i += 1
            while i < n:
                l = lines[i]
                if not l.strip():
                    # A blank line inside the list: the item goes on if the next line is indented under it.
                    k = i
                    while k < n and not lines[k].strip(): k += 1
                    if k < n and (len(lines[k]) - len(lines[k].lstrip())) > ind:
                        loose = loose or not LIST.match(lines[k])
                        body.extend([''] * (k - i)); i = k; continue
                    break
                lead = len(l) - len(l.lstrip())
                lm = LIST.match(l)
                if lm and lead <= ind: break
                body.append(l[min(lead, cont):] if lead > ind else l.strip())
                i += 1
            items.append(body)
        tag = 'ol' if ordered else 'ul'
        s = f' start="{start}"' if ordered and start != 1 else ''
        li = []
        for body in items:
            # The item's first lines are its text; an indented list under it is parsed as blocks.
            li.append('<li>' + self.blocks(body, tight=not loose) + '</li>')
        out.append(f'<{tag}{s}>' + ''.join(li) + f'</{tag}>')
        while i < n and not lines[i].strip():
            k = i
            while k < n and not lines[k].strip(): k += 1
            if k < n and LIST.match(lines[k]) and len(LIST.match(lines[k]).group(1)) == ind: i = k; break
            break
        return i
