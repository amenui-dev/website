# 材质与松开

材质是第二遍的画法：plain、dottr、halftonr、charr、smudgr、carvr、glitchr、near、scribblr。静止时它们和 plain 一样（除了自己有静止样子的几种）；松开时各有各的样子。`El::material(id)` 给一棵子树另一种材质。快速滚动时每种材质留下自己的痕迹（见 DESIGN §1.8.3）。

`scribblr` 静止时没有任何不同：它的第二遍就是 plain 的画面。松开时元素的填充淡出，墨变成一根等粗的线（`src/strand.rs`），作为形状画在第一遍里：先贴着字的真实轮廓，再用直的桥连成一笔，再松就游走、在场图有能量的地方打结、顺着流向甩出线头；收回时结先散、填充回来。要读的字只在轮廓上多一根线。`Look::beat`（`Beat { bpm, steps, swing }`，DESIGN.md §1.32）让线的姿势保持外观节拍的一步：180 bpm、每拍四步时一步是 83 ms，每秒 12 个姿势；`bpm` 为 0（默认）时不保持。`El::trace(Trace)` 在任何材质下按进度画一条长路径，实线或虚线，被场图的流向推歪，头经过处搅动场图。
