//! The site's colour tokens (AA.9), the same values as the shell's and the docs' CSS, and the greys
//! the canvas draws with until the framework's theme carries them.
use amenui::el::Paint;

/// One scheme's tokens, as written (sRGB, `0xrrggbb`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens { pub ground: u32, pub raised: u32, pub hairline: u32, pub dim: u32, pub text: u32, pub ink: u32, pub accent: u32, pub second: u32, pub warn: u32 }
/// Graphite: the dark scheme.
pub const DARK: Tokens = Tokens { ground: 0x1a1b1d, raised: 0x232528, hairline: 0x383b40, dim: 0x989da4, text: 0xd6d8d4, ink: 0xf2f2ee, accent: 0xf47a3c, second: 0x7db0f7, warn: 0xf2c94c };
/// Paper: the light scheme.
pub const LIGHT: Tokens = Tokens { ground: 0xf1f0eb, raised: 0xe6e5df, hairline: 0xcbc9c1, dim: 0x5f6266, text: 0x2b2c2e, ink: 0x141517, accent: 0xb23f0d, second: 0x1a5cc7, warn: 0x7d5c00 };
/// The tokens of a scheme.
pub const fn of(paper: bool) -> Tokens { if paper { LIGHT } else { DARK } }
/// A token as a theme colour (0–1).
pub const fn rgb(v: u32) -> [f32; 3] { [((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0] }

// The greys as the canvas draws them today. The framework's theme has a ground and an ink, and a grey
// is a tone between them, so `dim`, `text`, `hairline` and `raised`, which lie a little off that line,
// are drawn at the tone nearest them in both schemes (within 9, 3, 6 and 2 of the tokens per channel).
// When the theme carries the four greys, these go and the canvas takes `Tokens` through the theme.
/// Labels, numbers, hints, descriptions.
pub const DIM: Paint = Paint::tone(158.0 / 255.0);
/// Prose and idle control labels.
pub const TEXT: Paint = Paint::tone(225.0 / 255.0);
/// Card frames, rules, dividers, control outlines at rest.
pub const HAIRLINE: Paint = Paint::tone(43.0 / 255.0);
/// Stages and plates.
pub const RAISED: Paint = Paint::tone(12.0 / 255.0);
