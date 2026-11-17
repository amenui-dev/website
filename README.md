# amenui 官网

[amenui](https://github.com/amenui-dev/amenui) 的官网，全部由 amenui 自己画：打开就是首页 `#body`（形体，设计文档 §6.2）：一段文字聚成由它自己的字组成、慢慢转动、发光的形体，再回到能读的文字；画面里的小标记通到实验室（`#lab`）和组件库（`#gallery`）。另有七个展示场景（`#scenes`）、每个组件、记号、材质、风格的页面和文档（`docs/site/`，简体中文和英文）。许可证 MIT。

- `src/gallery/`：整个站点（顶栏、左栏、每一页），`src/demo.rs` 是骨架演示那一页。
- `examples/body.rs`、`examples/lab/`：首页的形体和实验室的每个实验，只用 amenui 的公开接口写（`website::showcase`）。
- `assets/i18n/`：站点自己的文字（amenui 的组件文字在框架里，`i18n::install` 把两份合在一起）。
- `web/`：网页外壳和打包脚本；`src/bin/`：原生窗口、样张、GPU 验收和计时工具；`tests/`：站点各页的测试。

```bash
cargo run --release --bin window              # 原生窗口，打开首页（也可以带页名，比如 `window button`）
```

## 运行

本仓库依赖 amenui 的 main 分支（`Cargo.toml`）。两边一起改时，把 amenui 放在旁边的目录，打开 `Cargo.toml` 末尾那段 `[patch]`。

测试（不需要显卡）：

```bash
cargo test
```

Gallery 的每一页在每个主题、每种材质下各出一张 PNG（文件名是 `页名-主题-材质.png`），另有对话框打开、切主题的中途、换页、输入、窄窗口几种状态（需要一个 Vulkan 驱动，没有显卡时可用 lavapipe）。参数是输出目录、宽、高（设备像素）和缩放：

```bash
cargo run --release --bin pages -- pages 1280 1800 1
```

材质的样张（每种材质松开度 0 / 0.5 / 1、标注页的四种模式、每个风格页静止和波前经过时，以及 `styles.png`、`materials.png` 两张拼版）；最后一个参数换成 `perf` 时量每种材质一帧的时间：

```bash
cargo run --release --bin pages -- materials 1280 1000 1 materials
```

每个组件页在每种材质（除了调试用的 `near`）下各出静止和波前经过预览中途两张，暗主题，print 和 lino 另出明主题；每种材质一张拼版 `components-<材质>.png`（每页一行：静止 | 波前）：

```bash
cargo run --release --bin pages -- final/components 1280 1100 1 components
```

展示场景的样张：每个场景一段脚本化的操作（合成的指针和按键）拍六个时刻，默认风格和另外两种风格各一张拼版，加 420 × 800 的窄窗口，和 `scenes.png`（每个场景最好看的一刻）、`scenes-list.png`（列表和标题流进场景）；可以只出一个场景（第二个参数是 0–6）；第二个参数是 `perf` 时量每个场景一帧的时间：

```bash
cargo run --release --bin scenes -- scenes
```

骨架演示的一组画面：

```bash
cargo run --release --bin shot -- shots 1280 900 1
```

读回像素检查设计文档 §1.6.1、§1.6.2、§1.6.3、§1.7 和 §1.10 的验收（浅色主题下完全松开的图片里没有页面底色，每种材质都是；点距 1 时等于原图；像素字边缘只有两种颜色；变形交接和落地的那一帧和松开的元素一样，飞行中有粒子；每种材质静止时正文和 plain 一样；标注的框正好在布局矩形上；绕开轮廓排的字离障碍物的墨至少 gap − 1 像素；每个场景静止时安静；内容重组的第一帧和最后一帧与旧字、新字的普通渲染逐像素相同；每个组件页在每种材质下静止时，控件文字的字形像素和 plain 一样，差别不超过它下面衬底的差别；开关、标签页、选项的选中和未选中在每种材质下都看得出来），失败时退出码不为 0（`check components` 只跑最后两条）：

```bash
cargo run --release --bin check
```

变形和换页的样张（每对 × 每种材质 × 七个时刻，和换页往返各五个时刻）；带 `perf` 时量粒子上限时的帧时间：

```bash
cargo run --release --bin film -- film
```

每张记号表、每种调色板、浮雕、分区、方向跟随、辉光、视差和对比擦除各出一张样张，另有记号图集本身和原图：

```bash
cargo run --release --bin sheet -- sheets
```

原生窗口（winit；macOS 上用 Metal，其他系统用 Vulkan）。可以带一个页名，直接打开那一页；`--shot 路径.png@毫秒` 在启动后那一刻把窗口自己的一帧读回存下（可以有多个），`--exit-after 毫秒` 到时关闭窗口；标准错误上报告显卡和每两秒画了多少帧：

```bash
cargo run --release --bin window -- dialog
```

在 Linux 上的验证方法（这里的容器没有显示器和显卡）：在 Xvfb 里运行，Vulkan 用 Mesa 的 lavapipe（llvmpipe，CPU），不需要设置 `WGPU_BACKEND`；winit 还需要 `libxkbcommon-x11-0`。用 `xdotool` 发真实的 X 输入：点左栏的“输入框”、点输入框、键入 “native window”、点顶栏“场景”、点“变形”卡片、点“下一步 →”、按 Esc；窗口在三个时刻读回自己的帧，同时用 ImageMagick 的 `import -window root` 从 X 服务器截屏对照，两者一致。1280 × 860 时约 25 帧每秒，退出码 0：

```bash
xvfb-run -a -s "-screen 0 1400x1000x24" ./drive.sh target/release/window out/
```

（`drive.sh` 是一段 xdotool 脚本，不在仓库里。）macOS 上只做了类型检查（`cargo check --target aarch64-apple-darwin`），没有实际运行过。

一帧和它的树（给模型和脚本用；页名是官网的地址或 amenui 的配方 `recipe-<名字>`）：

```bash
cargo run --release --bin render -- --page button --size 1280x800 --at 1000 --out out.png --tree out.json
```

涟漪和重组的三张样张（计数 1 → 2、0.3 秒连点 10 次、一次按下经过三种响应的邻居）：

```bash
cargo run --release --bin film -- film ripple
```

CPU 计时：按钮页按一次、一万行滚一格、5000 字绕开障碍物重排，各自的毫秒；然后用显卡（lavapipe 也行，只能相对比较）量粒子（5 万、20 万、100 万格）和只画变了的部分（悬停、插入点闪烁、按一次计数，整帧和按框）：

```bash
cargo run --release --bin bench
```

三维、粒子、过渡的样张（三维 × 每种材质、粒子、十二种过渡）：

```bash
cargo run --release --bin samples -- out mesh && cargo run --release --bin samples -- out particles && cargo run --release --bin samples -- out transitions
```

网页版（需要 `wasm32-unknown-unknown` 目标、同版本的 `wasm-bindgen-cli`、Python 的 fonttools）：

```bash
cargo build --release --lib --target wasm32-unknown-unknown
```

```bash
wasm-bindgen --target no-modules --no-typescript --remove-name-section --out-dir dist/pkg target/wasm32-unknown-unknown/release/website.wasm
```

```bash
python3 web/build.py
```

打开 `dist/index.html`（可以带页名，比如 `dist/index.html#dialog`）；`dist/artifact.html` 是同样内容、不带外壳标签的单文件。网页只有一个 canvas，负责把指针、滚轮、键盘、尺寸变化和地址里的页名转给框架。


## 发布

推到 main 以后，`.github/workflows/deploy.yml` 编出网页版，把 `index.html` 和 `embed-example.html` 放到 `gh-pages` 分支。GitHub Pages 和 Cloudflare Pages 都直接用这个分支，不需要再构建：GitHub Pages 选 “Deploy from a branch → gh-pages / (root)”；Cloudflare Pages 连这个仓库，生产分支选 `gh-pages`，构建命令留空，输出目录 `/`。
