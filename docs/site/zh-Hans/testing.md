# 用 render 和树测试

`render` 在固定的 60 Hz 时钟上播放一串事件，画出最后一帧和它的树；同样的参数给出逐字节相同的 PNG。

```sh
cargo run --release --bin render -- --page button --size 1280x800 --at 1000 --out out.png --tree out.json --locale en
```

`check` 在显卡上读回像素做验收（18 条，包括慢速滚动是整像素移动的图）。
