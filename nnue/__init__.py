"""Python tooling for training, exporting, and validating ZBra NNUE models.

PyTorch is intentionally not imported here.  It is an optional dependency used
only by training machines; model loading and inference belong to the Rust
engine.
"""

__all__ = ["__version__"]

__version__ = "0.1.0-alpha.1"
