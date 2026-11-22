# 1.18 3D

A 3D object is one more thing on the page (a fourth instance kind, `MESH`): it is drawn in the rect layout
gives it, in the current material, loosens, morphs and can be pressed; it is not a separate engine. Code:
`src/mesh.rs` (no GPU: matrices, generated meshes, a glTF subset, scene description, picking), `src/mesh.wgsl`
(the mesh pass and its resolve) and the renderer's `prepare_meshes`.

**Description.** `el::scene3d(Scene3d)` (100% wide and 240 tall by default) and `el::mesh(MeshId)` (one mesh,
default camera and light). `Scene3d { items: Vec<MeshInstances>, camera, light, spec: Mesh3d, picture }`:
`MeshInstances { mesh, transforms: Vec<Mat4>, paint, id }` is several copies of one mesh in one draw call (two
thousand low-poly instances are one page on the website); one camera (`Camera`: perspective `fov` or orthographic
`height`, `yaw`, `pitch` and `distance` about `target`, `near`, `far`); one directional light
(`Light { dir, ambient }`). Meshes live in `mesh::Library`: the built-in cube, UV sphere, icosahedron (0–3
subdivisions), torus, cylinder, cone and plane have fixed ids, and `Ui::add_mesh` adds others (generated in
code, or read with `mesh::load_gltf` behind the `gltf` feature: positions, normals, indices, node transforms;
no skinning, materials or textures). A mesh is a triangle list, each triangle using its face normal by default;
when `Mesh::normals` gives each corner its own normal the surface is smooth and only creases are edges
(`Mesh::creased(angle)` decides by the angle between neighbouring faces: smooth below it, creased above). More
generated primitives: `extrude` (a closed 2D outline extruded along z, both faces as triangle fans, side walls
smoothed by crease), `extrude_bored` (the same with a round hole), `tube` (washers, bushings), `gear` (a spur
gear: tooth count, pitch radius, thickness, hole; the tooth flank rises steeply from the root and curves to the
tip circle like an involute, with tooth thickness half the circular pitch at the pitch circle), `block` (a box
with every edge creased) and `rod` (a cylinder with smooth sides and creased ends). `Mesh3d` adds `tones` (2:
flat lit and shadow tones, with the terminator at `terminator`) and `outline` (lines on silhouettes and
creases, in logical px, in the ink or in the grey a group of copies asks for, `MeshInstances::outline`, by its
tone; the mesh pass keeps it in the id image's alpha and the resolve's, and a line takes the grey of the copy
nearest under it, read unfiltered); flat-shading normals and shading are interpolated in the fragment. Colour is a `Paint`
(tone, accent, hue), using theme colours like shapes do.

**The mesh pass.** Each `scene3d` element has a set of off-screen images the size of its device rect ×
`Mesh3d.resolution_scale` (0.5 by default, 0.25–2; above 1 the silhouette and the outline are supersampled and
fall on a pixel's share of them, so a turned edge is a smooth line: the card stack draws at 2), with 24-bit depth and three RGBA8 targets drawn at once
(multiple render targets work on WebGL2 without blending): shading (ambient + Lambert, flat), tone, accent and
hue; view-space normal xy and 16-bit linear depth; the instance index (+ 1, 16 bits) and normal z. Unchanged
elements (same `Scene3d::fingerprint` and size) are not redrawn. A pixel reads its colour and its outline's
grey from the texel under its centre, found from its whole position in the box, `floor((k + ½) × dims ÷ size)`
(not from interpolated coordinates, which at a whole scale land on a texel's edge and round either way by the
driver: one renderer lost an outline's top row and another its bottom). For a box of whole pixels this is exact
at every scale, so every driver draws the same pixels, and the element moved by whole pixels draws them again;
at 2 a pixel covers its own 2 × 2 texels and stands for the lower right one (the nearness is filtered over them)
(`tests/mesh_texel.rs`). The resolve's nearness is 8 bits over ±SPREAD, so a distance read from it is good to
about a tenth of a pixel (half a step stored, half filtered); its zero is on a whole step (128 of 255), and a
distance within one step of half a pixel and a whole number more is taken as that (`mesh_snap`), so an edge on a
pixel boundary covers the pixel inside it fully, the one outside not at all, and a 1 px outline is exactly its
grey with nothing beside it. A turned edge's distances are as read. To put a face's edges on pixel
boundaries at whatever size the element is drawn (a box zoomed by `El::fit`, any scale), `Mesh3d.snap` names a
rectangle of the plane z = 0: under an orthographic camera the projection is scaled and shifted along each axis
(`Scene3d::proj_in`, from the device rect the element is drawn in) so that its four edges land on the nearest
whole device pixels, under half a pixel each. Picks use the camera as it is (under half a pixel apart).

**The resolve.** Three small full-screen passes: `mask` (covered pixels are silhouette; a covered pixel whose
right or lower neighbour is a different instance, whose depth jumps more than `edge_depth` (relative depth,
0.04 by default) or whose normals differ by more than `edge_normal` (15° by default) is an edge, one pixel
wide), then `h` and `v` (an exact Euclidean distance: the nearest inside / outside horizontally, combined
vertically over ±R rows, R = SPREAD / device length per pixel + 1). The result is two signed distances, one for
the silhouette only and one for silhouette plus creases, and a projected normal code (1 facing the viewer, 2–9
one eighth of a half turn of direction each). In the G-buffer stage a `MESH` instance reads distance bilinearly
and colour by nearest: coverage comes from the silhouette distance (so the outline is smooth), `near.r` is the
silhouette-plus-creases distance, the coverage buffer uses the shape encoding (`r = shade × tone × (1 −
accent)`, `g = shade × accent`, the b code `hue × 0.15 + 0.07`: a shape, recognisable as a mesh), and `near.g`
holds the normal code (1–9 / 255, below every picture marking). So every material draws 3D without extra code:
creases are outlines in the SDF, `carvr` carves along them, `near` draws their iso-lines. `ambient` keeps the
darkest face off the page background. With `Scene3d.picture = Some(slot)` it is drawn as a picture (the marks of
§1.6.1, with its own ground).

**ASCII by normal**: in a mesh's cells, the `charr` material sets character density by shading; on creases
(`|sd| < 0.75 P`) it draws strokes along the crease, and on faces turned sideways strokes across the slope
(`| / - \`, the nearest of four families, one stroke `max(1, 0.14P)` wide), in the proportion
`ascii_normal_weight` (charr's 4th parameter, 1 by default).

**Marks by depth** (`Look::marks: Marks { depth_scale, depth_dim }`, both 0 by default, also in presets): the
mesh pass multiplies shading by `1 − depth_dim × far` and tone by `1 − depth_scale × far`, `far` being where this
instance's view-space depth lies between the scene's nearest and farthest (`Scene3d::depth_range`: each
instance's center plus and minus its bounding radius); materials that size and space marks by ink (`dottr`,
`halftonr`, `charr`, picture marks) therefore draw smaller and fainter far away. This is not a second route that
changes cell size by depth: the cells are still the one screen-wide grid (§1.6.1); what changes is the ink in
them. The 3D page's mechanism can use 0.45 and 0.35. `check` item 17 (formerly item 19): with both at 0 it is
byte-identical to off; turned on, more than 500 pixels far away darken and fewer than a tenth of that many
brighten.

**Interaction.** Picking happens on the CPU: the same transforms and camera cast a ray from the pointer
(bounding-sphere culling, then Möller–Trumbore), so headless, WebGL2 (where readback is asynchronous) and native
all behave the same, synchronously. `El::on_pick(|Pick| ..)`: `Pick::Hover(Option<MeshHit>)` when the pointer
moves to another instance (or leaves), `Pick::Press(..)` on press, with `MeshHit { object, instance }`, through
the existing hit-test path. `El::on_wheel` gets the wheel before scrolling does, except in the middle of a gesture
that is scrolling (§1.8.2), and leaves it to scrolling when it returns `None`. `widgets::orbit` is the camera
control: dragging orbits (computed from the angles at the press, `per_px` degrees per pixel), the wheel dollies
(`dolly`), the arrow keys turn once focused (`step_deg`, Shift × 4); on release it takes the velocity of the last
0.1 s and decelerates by the exact integral of `v e^(−friction t)` (like the fling in §1.8.2), stopping below
`stop`; the application calls `Orbit::step` in `on_tick` and wants frames with `on_wake` while `moving()`. The
camera is state; the component never moves by itself.

**Loosening.** 3D elements are decoration by default (`Response::Loosen`, not legible), even when draggable:
fronts and hover bursts loosen them through the same field (the burst is added to meshes in the G-buffer stage's
`own_loose`), and the scope rules (§1.15) apply as usual; the lens, as §1.15 says, touches only those the pointer
is not operating.

**Acceptance.** `tests/mesh.rs` (two thousand copies are one group in one draw; hover and press carry the
instance index and the hovered instance is drawn alone in the accent; dragging orbits and coasts to a stop with
no frames wanted after; the wheel and the arrow keys; fronts loosen meshes); unit tests in `mesh.rs`
(primitives are closed and face outward, the camera, picking the nearest, surface sampling, glTF); `check` item
14: drawing again after turning 0° and drawing with a new renderer are byte-identical; at resolutions 0.5 and 1
each row of a still silhouette is one run (convex objects), and under `near` every edge projected on the CPU
has an outline drawn on it (control: none with a threshold above every angle). Samples `samples <dir> mesh`:
the same object in every material at rest and loosened 0.7, plus four mark sets drawing it as a picture.
