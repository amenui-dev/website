# 1.20 语言

**Language is data.** Every string a person reads comes from a message catalog: one plain text file per
language (`assets/i18n/zh-Hans.txt`, `assets/i18n/en.txt`, `assets/i18n/ar.txt`), one `key = text` per line; `{name}` is a named
argument (numbers written the language's way, with digit grouping), `{{` is a brace; a line starting with two
spaces continues the previous entry (a newline); `key.one`, `key.other`… are plural forms, chosen by the
argument `n` (or the first number) with CLDR's cardinal rules (`intl_pluralrules`); Chinese needs only `other`,
English `one` and `other`, Arabic all six (`Locale::categories` asks the rules which). Fluent is not used: `fluent-bundle` brings `intl-memoizer`, `fluent-syntax` and a type
system, and this needs only key → text, arguments and plurals, so the parser is 60 lines of our own. Catalogs are
compiled in with `include_str!`, so the web version has them too.

**Interface** (`src/i18n.rs`): `Locale` (`ZhHans`, `En`, and `Other(Tag)` for any other BCP 47 tag, with
`Locale::AR` shipped; `tag`, `from_tag`, `name`, `direction`, `fallback`,
`plural`, `number`, `date`); `locale.tr(key)`, `locale.tr_args(key, args)`, `t!(locale, "key", n = 3,
name = "…")`; `try_tr` returns a `Result`, a missing key being `I18nError::MissingKey`, which names the key, the
chain of languages searched (zh-Hans → en) and which line to add to which two files; `tr` shows `⟨key⟩` when
nothing is found and records it (`i18n::misses()`, which tests use to list what is missing). The fallback chain:
keys missing in any language but English use English. A language that is not built in may give its own
name (`locale.name`) and date pattern (`locale.date`, else ISO 8601). `from_tag` accepts any well-formed tag,
matching a shipped or installed catalog by tag or by language (`ar-EG` → `ar`). An application installs its own catalogs with
`i18n::install`, looked up before the framework's.

**Language lives in the state.** An application keeps its `Locale` in its state, and changing language is an
ordinary message (the website's `Msg::Locale`); `Ui::on_locale(fn(&S) -> Locale)` lets the runtime set it as the
current language (`i18n::current`) before every `view`, and components get their own strings (the dialog's
"Close", a picture's name without a caption, the accessible names of a number control's side buttons, the 3D view,
the ten-thousand-row list's "n rows") with `i18n::tr`. After a language change the whole page is laid out again;
the text layout cache is keyed by the string itself, so nothing stale is used; keyed text recomposes as usual (the
pixels of the changed characters fly into the new text, §1.16); text without a key is replaced and condenses
from loose. The website's top bar has a Language menu (beside theme and material), and the marking at the top
right of Body (English only, `LANGUAGE  EN` / `ZH`) changes it on press; tools use `AMENUI_LOCALE=en` and
`render --locale en`.

**Direction.** `El::rtl(true)` runs a subtree right to left (inherited): taffy's `Style::direction`, so a row's
first child is at the right, start alignment is the right and padding and margins mirror. Bidirectional text in
a paragraph is laid out by parley with the Unicode bidirectional algorithm. **Not covered**: text alignment does
not follow `rtl` (a paragraph's start is decided by parley from the text's own direction); caret movement and
selection in a text field have not been specifically verified in RTL text; scroll bars, annotation leaders and
the website's side rail are not mirrored; `Locale::direction` is right to left for
Arabic, Hebrew, Persian, Urdu and the other right-to-left languages (or an Arabic, Hebrew, Thaana, Syriac, N'Ko,
Adlam or Hanifi script subtag). `tests/i18n.rs` checks that Arabic and Hebrew paragraphs get a glyph for every
character (with a fallback face) and that the caret before the first character is right of the one after the
last.

**Acceptance**: unit tests in `i18n.rs` (parsing, continuation lines, errors for duplicate keys and bad lines,
arguments, digit grouping, plurals, dates, the fallback chain and the missing-key error); `tests/i18n.rs`: every
key exists in every language (plural keys have every form the language needs; Body's `body.*` keys only in the
English catalog); the Body landing scene in both languages (text and body states) has no Han character in its
tree and no missing keys (that page is English only); a right-to-left row has its first child at the right and
starts from the right.
