# 预设

一个风格预设是数据：主题、材质和它的参数、标注、交互的性格、滚动的样子和一段过渡程序。`Preset::apply` 给出外观；`Preset::set_number("transition.steps.0.a", 3.0)` 改其中一个数。

一共十个。第十个 Loose Thread 是黑白加一种强调色、`scribblr` 材质：静止时是干净的字，指针搅过场图的地方出线，悬停时有边注和角标，700 ms 的过渡先松成线再先画轮廓。收尾的一遍为它加了两个开关：`post.binary`（只用黑白）和 `post.invert`（翻转）。
