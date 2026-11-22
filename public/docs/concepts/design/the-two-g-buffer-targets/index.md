# 1.4 The two G-buffer targets

- The coverage buffer (`ink`, RGBA8): what is here. Text and shapes write tone in r, accent in g and a
  code in b (a pass-through value b / a): below 0.45 it is `(hue + 3 × is_shape) / 14`. Pictures write
  their colour, with coverage in a.
- The SDF buffer (`near`, RGBA8, max-blended): r is the **signed** distance, 0.5 on the outline, rising
  to 1 at `SPREAD` px inside and falling to 0 at `SPREAD` px outside (the glyph atlas stores exactly
  this, from exact Euclidean distance transforms inside and outside; shapes compute it analytically).
  Taking the maximum is exactly the union of distance fields. g says whether this is a picture, storing
  `(slot + 1) / 8` if so. b is the instance's own looseness (set by birth, a press or at rest) plus the
  fronts and lens that reach it, up to its cap (§1.15). Looseness belongs to the instance's own ink:
  where there is ink (coverage over half) it writes `128 + 127k`, in the distance band outside the ink
  `125k` (eight bits), so under max blending any ink wins over anyone else's distance band and one
  instance's looseness never leaks onto another's ink (a hover line appearing on a chart used to dim and
  loosen the three columns of dots beside it). The shader decodes it with `loose_t(p)`. a is the material
  override (at or above 128/255 it is `128 + depth × 16 + material`, depth being how deeply overrides are
  nested, so under max blending the innermost override wins), otherwise a glyph's print pitch × 2 / 255
  (0 means not printed).

WebGL2 cannot blend each target differently, so the same geometry is drawn twice instead of into
multiple targets.

Instances in a layer write the SDF buffer without blending and only where they have ink: inside their
own outline they replace the looseness and picture marking beneath, so a dialog itself does not loosen
and a picture beneath a card does not turn into marks through it.
