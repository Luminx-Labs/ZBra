# NNUE Python tooling

This package contains Python-side model tooling: training adapters, export,
metadata validation, and integration helpers.

It is not the engine runtime. The production evaluator lives in
`crates/nnue` and must consume exported `.nnue` files without importing
Python, PyTorch, or NumPy.

PyTorch is available only through the optional `training` dependency extra.
