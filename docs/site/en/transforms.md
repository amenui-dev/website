# Transforms

Two kinds: a morph (`Cmd::Morph`: one element breaks into cells that fly onto another's shape) and a flow (`Cmd::Flow`: what two pages share moves to its new place on a page change). Both go through the same two passes; the material draws what is in flight as usual.
