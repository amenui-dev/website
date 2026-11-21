# 概览
## 树和消息


每个消息之后，运行时调一次 `view(&state)`，得到一棵新的 `El` 树，按 key 和上一棵比较，只重算变了的布局。状态是唯一的来源；组件从不自己动。

### 命令

要运行时做的事（变形、流动、聚焦、滚到某处）放进状态里的命令，用 `Ui::on_commands` 交出去：`Cmd::Morph`、`Cmd::Flow`、`Cmd::Focus`、`Cmd::ScrollTo`、`Cmd::Commit`。


## 布局


布局是 taffy 的弹性盒，单位是逻辑像素；`stack()` 把子元素叠在一起，自身大小不变；元素默认不收缩。`row()`、`col()`、`stack()`、`space(w, h)`；`.gap`、`.pad`、`.grow`、`.fill_width`、`.scroll()`。`El::rtl(true)` 让一棵子树从右往左。`El::zoom(k)` 把一棵子树里的所有长度乘 `k`。


## 涟漪规则


按下放出一道波前，默认只到被按的控件自己（`Scope::Own`）；`Scope::Container`、`Scope::Global` 放得更远。每个元素有自己的响应：`Ignore`、`Nudge`、`Loosen`。要读的字和控件最多松开到 `Look::legible_cap`。


## 变换


两种：变形（`Cmd::Morph`：一个元素散成格子，飞到另一个元素的形状上）和流动（`Cmd::Flow`：换页时两页共有的部分移到新位置）。都在同一套两遍绘制里，材质照常处理飞着的东西。


## 预设


一个风格预设是数据：主题、材质和它的参数、标注、交互的性格、滚动的样子和一段过渡程序。`Preset::apply` 给出外观；`Preset::set_number("transition.steps.0.a", 3.0)` 改其中一个数。

一共十个。第十个 Loose Thread 是黑白加一种强调色、`scribblr` 材质：静止时是干净的字，指针搅过场图的地方出线，悬停时有边注和角标，700 ms 的过渡先松成线再先画轮廓。收尾的一遍为它加了两个开关：`post.binary`（只用黑白）和 `post.invert`（翻转）。


## 三维


`el::scene3d(Scene3d)` 是页面上的又一种元素：网格在离屏图里画出明暗、法线和深度，再变成和字一样的墨与近度，所以每种材质都能画它。生成的基本形有齿轮、挤出的轮廓、带折痕的长方体和圆柱；`Mesh3d::tones = 2` 是两种平涂色调，`outline` 画出轮廓线。看[三维页](#three)。


## 粒子


变形的格子多时在显卡上模拟（计算着色器或浮点目标的乒乓），少时在 CPU 上；网格一边的格子从它的表面出发。`Look::particles.gpu_above` 决定从哪里开始用显卡。
