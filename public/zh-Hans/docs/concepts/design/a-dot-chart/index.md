# 1.25 点阵图表

`widgets::dot_chart` (`DotChart { knots (7, 0..1), cols 56, rows 16, pitch 9, range (0.42, 0.68), max 120,
unit }`). The curve is a Catmull-Rom spline through the control points, sampled once per column as v(c). Dot r of
column c (counting up from the axis) covers the band [r, r+1)/rows, and how much of it lies under the curve is its
coverage `k = clamp(v·rows − r, 0, 1)`: diameter `pitch × (0.25 + 0.65k)`, tone `0.25 + 0.75k`, so a column's
coverages add up to exactly v·rows and the field of dots is the area under the curve; above the curve are the
smallest, faintest dots. The picked range is ink: its dots' tones go up to the ink, with an ink line under the axis;
outside it a dot's tone is scaled to go no further than dim (`tone × Paint::DIM.tone`). The accent is only under the
hand: the dot under the pointer, and a control point and its tick while it is dragged. While the pointer
moves over the chart (`El::on_move`), that column shows a vertical line and its value (in the mono face, which is the
pixel face); each control point on the axis is a draggable round handle (`Role::Slider`), dragged up and down to
change its value, with the curve and dots recomputed; once focused, Left and Right move the line and Up and Down
change the nearest control point. The formula caption prints the expressions for k, diameter and tone and the
range's area. Acceptance: unit tests in `dot_chart.rs` (sums of coverage, the spline passes through its control
points, dragging); `tests/chart.rs` (dot size by coverage, ±0.5 because layout rounds; the line and value when
sweeping; dragging a control point raises the curve and enlarges that dot; the line disappears on leave). Samples
`craft <dir> chart`.
