//! The site in a native window (`amenui::app::native`): Metal on macOS, Vulkan elsewhere.
//! usage: cargo run --release --bin window [page] [--shot <path.png>@<ms> …] [--exit-after <ms>]
//!   page: a hash name, e.g. dialog. `--shot` reads the window's own frame back at that many ms after
//!   start and saves it (any number of them); `--exit-after` closes the window after that many ms.
//!   It reports the adapter and, every two seconds, how many frames it drew, on stderr.
fn main() -> std::process::ExitCode {
    let mut window = amenui::app::Window::new("amenui").from_args();
    window.report = true;
    match amenui::app::native::run(window, website::setup, website::make) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => { eprintln!("{e}"); std::process::ExitCode::FAILURE }
    }
}
