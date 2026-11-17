# Annotation

The third pass is the annotation layer: frames, numbers, sizes, leaders and rulers drawn from the real layout, never loosened. `El::annotate(text)` writes a callout; `{xy}` becomes the element's centre.

Three kinds read the whole scope rather than an element: edge notes (`EDGE`: a hairline on the scope's right edge with ticks, and small serif notes turned to read upwards giving its size, the pointer, the material and theme, the loosest element and the field map's non-zero nodes), a waveform (`OSC`: the field map's energy along one row, one device pixel thick, `osc_amp` and `osc_y`) and a level bar (`LEVEL`: the mean energy against `level_full`). `Ui::readout()` says what they drew; the exported tree's root carries `field: { level, nonzero }`.
