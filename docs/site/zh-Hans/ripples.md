# 涟漪规则

按下放出一道波前，默认只到被按的控件自己（`Scope::Own`）；`Scope::Container`、`Scope::Global` 放得更远。每个元素有自己的响应：`Ignore`、`Nudge`、`Loosen`。要读的字和控件最多松开到 `Look::legible_cap`。
