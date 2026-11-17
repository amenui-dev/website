# amenui website

The site of amenui (https://github.com/amenui-dev/amenui), built with amenui itself: Rust compiled to wasm, drawn on a canvas, with a small HTML shell per address. Deployed from `main` to the `gh-pages` branch by `.github/workflows/deploy.yml` (base path is a build variable, `/website/` today).

## How work is split

- Every design and aesthetic decision (layout, type, colour, motion, copy, interaction, information architecture) is made by Fable 5.2 as the design advisor. Implementation is done by Opus 5.5. The session that coordinates reviews renders against the design and sends before/after images back to the advisor for sign-off; it does not make taste calls itself.
- The design of record is `/mnt/project-files/amenui/website/site-design.md` in the project's shared files (sections A–G and later). Read it before changing anything visible.
- The docs text (Markdown under `docs/` here and in amenui, and `README.md`) is owned by the docs work; the site owns how those pages are laid out and reached.

## Rules

- Commit straight to `main`, no pull requests. Messages are `<scope>: <what changed>` and describe the content, never rounds, steps or the process that produced them. Commits are signed.
- On-screen text in demos, scenes and the home stage is English only. The site chrome and docs keep i18n (`en` and `zh-Hans`, `website::LOCALES`, catalogs in `assets/i18n`).
- MIT license.
- Use standard graphics terms in comments and docs (coverage buffer, SDF, G-buffer, deferred shading), not in-house names.
- Framework changes belong in amenui; the site depends on amenui `main` by git (uncomment the `[patch]` in `Cargo.toml` to work on both).

## Commands

- `cargo build --profile quick --bin render` then `target/quick/render --page <hash> --size 1440x900 --locale en --at 3000 --out x.png` renders any page offscreen (needs a GPU or Mesa's lavapipe: `apt-get install mesa-vulkan-drivers`).
- `cargo test --profile quick`, `cargo clippy` native and `--target wasm32-unknown-unknown` with `-D warnings`.
- Web build: see `.github/workflows/deploy.yml` (wasm-bindgen at the Cargo.lock version, then `python3 web/build.py`).
