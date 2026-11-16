# Performance

Draw only when something changed; draw only what changed (two boxes); scrolling moves, it does not lay out again; a long list builds only the rows that show; the material can shade loose pixels at half resolution.

## The readout

F2 (or the Performance menu in the gallery's top bar) opens the readout; again shows every number; a third time closes it. It is drawn by the annotation layer at the bottom right in the monospaced pixel face. It changes only when a frame is drawn anyway, and once more a second after the last frame, to say `idle`; then it asks for no frames.

| Number | Meaning |
|---|---|
| fps | The instant rate from the last two frames' interval, then the frames really drawn in the last second. `idle` at rest. |
| frames | Frames drawn, skipped because nothing changed, and dropped (an interval over 1.5 frames at 60 Hz while frames come one after another). |
| update+layout | Handling messages, building the view and laying out (ms: last frame, mean over a second, largest in it). |
| of it text | Of that, shaping and breaking text (parley). |
| scene | Building the instances from the laid-out nodes. |
| gpu submit | Recording and submitting the GPU work (on the CPU). |
| sparkline | The last 120 frames, one bar each, as tall as it cost, full height 16.7 ms; stacked from the bottom: update and layout, scene, submit (in the accent). |
| dirty | The share of the screen this frame redrew (the union of the two boxes), and the render passes recorded. |
| instances | Glyph, shape, picture and 3D view instances. |
| particles | Particles in flight, and their path: compute shader, float textures, or the CPU. |
| tree | Nodes in all, nodes that changed in this view, layout passes so far. |
| text | Paragraphs in the cache, paragraphs shaped so far, how full the glyph atlas is. |
| textures | An estimate of the renderer's texture memory (the full-screen images and the glyph atlas), and the fronts travelling. |
| look | The material, the quality (the material pass's resolution) and whether it is automatic. |
| device | The graphics backend (WebGPU, WebGL2, Vulkan, Metal…) and the adapter's name. |
| surface | The window's logical size, the device pixel ratio, the language. |

The one-line mode shows fps, the total cost, the share redrawn, the instance count, the material and the backend.
