# 1.9 平台

- Native: a winit window (`app::native::run`, Metal on macOS, Vulkan elsewhere); off-screen rendering to
  PNG (`shot`, `pages`) for acceptance.
- Both shells tell the application (`app::App`) about the system's appearance (`set_appearance`: dark,
  and more contrast where the platform says so: the web's `prefers-contrast: more`; winit has no such
  setting, so native windows report `false`), at start and on every change, and about files dragged over
  and dropped (`files_hovering`, `files_dropped`: a path natively, the name and bytes on the web, which
  reads every file of a drop before handing them over together; winit reports one file per event).
- Native: when the GPU device is lost (a driver reset or update, an external GPU unplugged), the device's
  lost callback marks it and the next redraw requests a new adapter and device on the same surface,
  builds a new renderer, runs the application's `setup` again (so `setup` is `Fn`), and marks the glyph
  atlas for upload; mesh buffers and particle tiles upload again because the new renderer's counters start
  at zero. The `Ui`, and with it the state, focus and scroll positions, is untouched. A device dropped on
  purpose (`DeviceLostReason::Destroyed`) is not a loss. The web shell does not recover yet: a lost WebGPU
  device leaves the canvas still until the page reloads.
- Web: one canvas, drawn only when needed (§1.11). The web page only forwards pointer, wheel (both
  directions) and keyboard events and `location.hash`: at start-up it hands the hash to the application, on
  `hashchange` (links, back) it switches to that page, and after every input it writes the current page's
  name back to the hash. Page names are lowercase letters, digits and hyphens (`#button`, `#marks-chars`).
  Input methods and the clipboard are in §1.8.1.
- Web: a panic does not stop the page. The module has no unwinding, so a panic traps the call it happens in and
  nothing that call held is released; the running application is kept in a `app::slot::Slot` that holds no
  borrow across a call, and the panic hook (`app::web::panic_hook`) marks the panic, so the next call goes on
  with the application as the panic left it (and says so on the console) instead of failing for good with
  "already borrowed". A GPU error the browser reports (a buffer the adapter refuses) is logged, not a panic. The
  mesh pass puts the library's vertices in buffers of at most 256 KB (`render::MESH_CHUNK`, cut at whole
  triangles), because some adapters refuse a buffer under a megabyte (SwiftShader refused 760 176 bytes).
- Both hosts start with `Renderer::begin` and warm the renderer up after each frame they draw
  (`Renderer::warm_frame(ui, WARM_MS)`, 6 ms, §1.23), so the first frame is the plain page and the material arrives a
  few frames later. The runtime is told how far it is (`Ui::warm`, `Ui::on_warm` for the state). A page at rest
  draws nothing, so the warm-up does not wait for frames: the hosts also step it in idle time
  (`Renderer::warm_idle`), which draws nothing and asks for no frame except the one after which the look's material
  can be drawn (the page then condenses into it). The native host does it before it waits, coming straight back while
  pieces are left; on the web the exported `warm_idle(budget_ms, t)` (1 while pieces are left, plus 2 when a frame is
  wanted; 1 too when the module was busy) is called by `web/amenui.js` from `requestIdleCallback` (an animation frame
  that does only warm-up work where there is none), and the next idle callback is asked for whatever the call did
  (a piece that blocked for long, a call that threw) until the warm-up says it is done. A page that asks for no frames after its first is warm within a bounded time of quiet, and
  later input frames make no pieces. On the web the exported `warm()` returns `[done, total]` (empty once warm), and
  `web/amenui.js` sets `aria-busy` on the canvas while it warms and calls `opts.onWarm(done, total)` each time it
  moves on.
- A frame drawn while something is moving owes one more frame: when a motion ends between two frames, the next
  frame draws the settled page rather than leaving the last one mid-motion (`Ui::needs_frame` and `next_wake` count
  it; §1.11).
