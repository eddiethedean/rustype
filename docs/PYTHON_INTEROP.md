# Python Interoperability

Python interoperability is a primary product requirement, not a secondary compatibility feature.

Rustype succeeds only if `.rpy` code can use the existing Python ecosystem with minimal friction.

## Importing Python from Rustype

Normal Python imports should remain valid:

```python
import polars as pl
from pydantic import BaseModel
from fastapi import FastAPI
```

The compiler should preserve ordinary import semantics whenever possible.

## Calling typed Python

For typed Python packages, Rustype should consume available annotations and stub information as interoperability metadata.

Typed Python APIs may be called directly when their contracts can be mapped safely into the Rustype type model.

## Calling untyped Python

Untyped Python is a static-safety boundary.

Calls whose return values or side effects cannot be proven should produce dynamic/unknown values or require explicit `unsafe` treatment depending on the operation.

Rustype should not silently treat an untyped dependency as fully trusted.

## Python `None` and Rustype `Option`

Python APIs commonly encode optionality as `T | None`. Rustype should provide predictable adaptation between those APIs and `Option[T]`.

The initial compiler may insert or expose explicit conversion helpers rather than automatically converting every nullable Python expression.

The language specification should distinguish:

- Python runtime `None`
- Rustype's `Option` empty variant

Even if their lowered representation overlaps.

## Exceptions and `Result`

Python dependencies generally signal recoverable failure through exceptions rather than `Result`.

Rustype should not magically infer every exception a Python function may raise. Interop should instead provide explicit adaptation patterns.

Example conceptual helper:

```python
fn read_config(path: str) -> Result[str, IOError]:
    return Result.catch(IOError, lambda: Path(path).read_text())
```

Framework-specific adapters can evolve later.

## Python classes

Rustype should interoperate with normal Python classes, dataclasses, Pydantic models, SQLAlchemy models, framework objects, and metaclass-driven systems as far as their behavior is statically representable.

Highly dynamic class behavior may require an unsafe boundary.

## Decorators

Decorators are essential to frameworks such as FastAPI, pytest, Click/Typer, SQLAlchemy, and many others.

Rustype must support decorators on Rustype functions and classes where their transformed type is known or conservatively representable.

Unknown decorators should not silently destroy type guarantees.

## Generated Python API

Rustype-generated modules should expose ordinary Python objects so `.py` code can import them.

```text
models.rpy -> generated models.py
```

A Python module should be able to consume public Rustype exports without needing a Rustype interpreter.

## Build/import integration

Potential approaches include:

- compile `.rpy` to a build directory before execution,
- editable-install/build-backend integration,
- a development import hook,
- `rustype run` that compiles and executes transparently.

The stable deployment path should favor explicit/reproducible compilation rather than runtime compilation magic.

## Packaging

Rustype projects should eventually integrate with standard Python packaging and tools such as `pyproject.toml`, `uv`, `pip`, wheels, and editable installs.

The package ecosystem should remain PyPI; Rustype should not create a parallel registry.

## Runtime compatibility

Generated code should target supported CPython versions explicitly. The compiler should have a configured Python target, for example:

```toml
[tool.rustype]
python = "3.12"
```

The backend can then generate syntax and typing constructs appropriate for that target.

## Pydantic

Pydantic is a natural optional integration for turning untrusted runtime data into typed application values.

Rustype should recognize validation conceptually without making Pydantic a core dependency.

Future adapters may understand `BaseModel.model_validate`, `TypeAdapter`, or registered validation functions as trusted boundary transitions.

## FFI boundary principle

Python itself is effectively Rustype's foreign environment.

The guiding rule is:

> Python interoperability should be easy, but loss of Rustype guarantees should be explicit.
