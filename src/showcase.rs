//! The showcase pieces, written as examples against amenui's public API: the Body scene
//! (`examples/body.rs`, the landing page) and the Lab's experiments (`examples/lab/`).
#[path = "../examples/lab/lumen.rs"] pub mod lab_lumen;
#[path = "../examples/lab/contours.rs"] pub mod lab_contours;
#[path = "../examples/lab/silk.rs"] pub mod lab_silk;
#[path = "../examples/lab/grown.rs"] pub mod lab_grown;
#[path = "../examples/lab/mercury.rs"] pub mod lab_mercury;
#[path = "../examples/lab/flow.rs"] pub mod lab_flow;
#[path = "../examples/lab/loupe.rs"] pub mod lab_loupe;
#[path = "../examples/lab/orbit.rs"] pub mod lab_orbit;
#[path = "../examples/lab/spring.rs"] pub mod lab_spring;
#[path = "../examples/lab/stack.rs"] pub mod lab_stack;
#[path = "../examples/body.rs"] pub mod scene_body;
/// The showcase scenes written as examples against the public API (DESIGN.md §6.2): name and source.
pub mod scenes {
    pub use super::scene_body as body;
    pub const ALL: [(&str, &str); 1] = [("body", include_str!("../examples/body.rs"))];
}
/// The Lab's experiments (DESIGN.md §5.3), one file each in `examples/lab/`, in page order: name and source.
pub mod lab {
    pub use super::{lab_lumen as lumen, lab_contours as contours, lab_silk as silk, lab_grown as grown, lab_mercury as mercury, lab_flow as flow, lab_loupe as loupe, lab_orbit as orbit, lab_spring as spring, lab_stack as stack};
    pub const ALL: [(&str, &str); 10] = [
        ("lumen", include_str!("../examples/lab/lumen.rs")),
        ("contours", include_str!("../examples/lab/contours.rs")),
        ("silk", include_str!("../examples/lab/silk.rs")),
        ("grown", include_str!("../examples/lab/grown.rs")),
        ("mercury", include_str!("../examples/lab/mercury.rs")),
        ("flow", include_str!("../examples/lab/flow.rs")),
        ("loupe", include_str!("../examples/lab/loupe.rs")),
        ("orbit", include_str!("../examples/lab/orbit.rs")),
        ("spring", include_str!("../examples/lab/spring.rs")),
        ("stack", include_str!("../examples/lab/stack.rs")),
    ];
}
