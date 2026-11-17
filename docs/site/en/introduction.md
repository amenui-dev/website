# Introduction

amenui is an interface framework that draws every pixel itself, written in Rust on wgpu; the same code runs in a native window and on the web. Everything on screen — glyphs, shapes, pictures, 3D meshes — is an instance with a true outline that can loosen, morph and close again along it; how it looks is a material you can swap.

You write a view (state → element tree) and an update (state + message → new state); the runtime does layout, input, animation and drawing.

## This documentation

- [Getting started](#docs-installation): install, write a first view, find your way around.
- [Concepts](#docs-tree-and-messages): the tree and messages, layout, text, materials, ripples, transforms, presets, annotation, 3D, particles, languages, accessibility, performance, testing.
- [Components](#button): one page each — preview, code, installation, usage, examples, API, keyboard and accessibility, and how it looks under every material.
- [For AI](#ai-agents): the entry point for models writing interfaces, recipes and the component registry.
