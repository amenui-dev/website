# Languages


Every string on screen comes from a message catalog, one per language, so an application can ship in any language without touching its views.

## Catalogs

A catalog is a plain text file (`assets/i18n/en.txt`, `zh-Hans.txt`, …): one `key = text` per line, `{name}` for an argument, and `key.one` / `key.other` (and the other CLDR plural categories a language uses) for plural forms. The framework ships catalogs for its own components in English, Simplified Chinese and Arabic; an application installs its own once at start with `i18n::install`, and they are looked up before the framework's.

```rust
use amenui::i18n::Locale;
let l = Locale::En;
assert_eq!(amenui::t!(l, "widget.list.rows", n = 3), "3 rows");
```

## Locales

`Locale::En` and `Locale::ZhHans` are built in; `Locale::Other` takes any BCP 47 tag (`ar`, `pt-BR`, `sr-Latn`), made with `Locale::from_tag`. A key missing from a catalog falls back to English, and `i18n::misses` lists the keys that did, so a test can fail on them. `Locale::number` and `Locale::date` format numbers and dates for the language.

The locale lives in the application's state. `Ui::on_locale` makes it current while the view is built, so `i18n::tr` reads it without passing it around; switching language is an ordinary message and the next frame is drawn in the new one.

## Scripts and direction

A locale that reads right to left, such as Arabic, lays rows out from the right ([Right to left](#docs-rtl)). Text is shaped by parley with the bundled faces first and the system's fonts after them, so a script the bundled faces don't cover still draws.
