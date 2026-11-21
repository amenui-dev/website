# 字体



The fonts in `assets/` are all under the SIL Open Font License 1.1. Their licenses are in
`assets/licenses/`. `text::embedded()` returns them compiled into the binary (about 14 MB, what a
native application ships with; the native window uses it); `text::load_bundled(dir)` reads them from a
folder instead, such as `text::ASSETS` in a checkout. The web build fetches the three main faces and
compiles in only Geist Pixel and the serif.

| File | Face | Source | Used as |
| --- | --- | --- | --- |
| `NotoSansSC-Regular.otf` | Noto Sans SC | Google Noto | The outline face for Latin, Han and kana |
| `DepartureMono-Regular.otf` | Departure Mono 1.500 (Helena Zhang) | Its release package | The Latin pixel face, 11 px |
| `FusionPixel-12px-Proportional-zh_hans.otf` | Fusion Pixel 12px, proportional, Simplified Chinese (TakWolf, upstream 2026.08.11) | See below | The CJK pixel face, 12 px |
| `GeistPixel-{Square,Grid,Circle,Triangle,Line}.otf` | Geist Pixel (Vercel and basement.studio) | npm package `geist` 1.9.2 | Optional Latin pixel faces (`text::GEIST_PIXEL`) |
| `SourceSerif4-8pt-subset.ttf` | Source Serif 4 (Frank Grießhammer, Adobe) | Google Fonts repository | `Font::Whisper`, small lowercase type and margin notes (`text::SERIF`) |

Pixel faces are drawn only at integer multiples of their size, at whole-pixel positions and without
anti-aliasing, so their edges hold only ink and ground. Any face can also be printed as pixels through
the material (`.print(pitch)`).

## How each file was made

- **Fusion Pixel** was merged back into one OTF from the WOFF2 files split by Unicode block in the npm
  package `@vp-tw/cjk-web-fonts-fusion-pixel-font`. The licenses of the upstream fonts it includes
  (Ark Pixel, Cubic 11, Galmuri) are kept in `assets/licenses/fusion-pixel/`.
- **Geist Pixel** was converted from WOFF2 to an OTF container with fontTools, with no change to the
  glyphs or names. Its license field reads "SIL OPEN FONT LICENSE"; the text is in
  `assets/licenses/geist-pixel/OFL.txt`. It has Latin only.
- **Source Serif 4** was instanced from the variable font `SourceSerif4[opsz,wght].ttf` with fontTools at
  the 8 pt optical size and weight 400, the cut drawn for small sizes, and subset to Basic Latin,
  Latin-1 Supplement and common punctuation (82 KB). The glyphs are unchanged. The license is in
  `assets/licenses/source-serif/OFL.txt`; it has no reserved font name.

## Coverage

The bundled faces cover Latin, Han and kana (and Hangul in Fusion Pixel). For other scripts (Arabic,
Hebrew, Thai, Devanagari, emoji…) a `Ui` takes more faces:

- `Face::fallback(bytes)`: tried after the face asked for and the first face, in the order given. This is
  the way on the web, where there are no system fonts.
- `Face::system()`, last in the list: the system's installed fonts, after every given face (feature
  `system-fonts`, on by default; fontconfig is loaded at run time on Linux). `app::native::run` adds it.

The bundled faces still decide every character they have, so text in Latin and Chinese looks the same on
every machine; tests and `offscreen` do not add the system's fonts, so their pixels do not depend on the
machine. Thai, Lao, Khmer and Burmese line breaking needs the `complex-scripts` feature (dictionaries of a
few MB). `Text::missing_glyphs` counts characters no face has.
