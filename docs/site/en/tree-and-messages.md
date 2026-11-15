# The tree and messages

After every message the runtime calls `view(&state)` once, gets a new `El` tree, compares it with the last by key and lays out again only what changed. The state is the only source; components never move by themselves.

## Commands

What the runtime should do (a morph, a flow, focus, scroll somewhere) goes into commands kept in the state and handed over with `Ui::on_commands`: `Cmd::Morph`, `Cmd::Flow`, `Cmd::Focus`, `Cmd::ScrollTo`, `Cmd::Commit`.
