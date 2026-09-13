"""Minimal runtime ABI scaffolding for generated Rustype Python.

The runtime surface is intentionally tiny during the compiler bootstrap. Stable
Option/Result representations will be added when the algebraic-value slice is
implemented.
"""

RUNTIME_ABI = 0

__all__ = ["RUNTIME_ABI"]
