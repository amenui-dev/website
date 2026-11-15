# 树和消息

每个消息之后，运行时调一次 `view(&state)`，得到一棵新的 `El` 树，按 key 和上一棵比较，只重算变了的布局。状态是唯一的来源；组件从不自己动。

## 命令

要运行时做的事（变形、流动、聚焦、滚到某处）放进状态里的命令，用 `Ui::on_commands` 交出去：`Cmd::Morph`、`Cmd::Flow`、`Cmd::Focus`、`Cmd::ScrollTo`、`Cmd::Commit`。
