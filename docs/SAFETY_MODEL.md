# Safety Model

Rustype uses the word **safety** to mean preservation of its static type and control-flow guarantees. It does not claim Rust-style memory safety.

## Safe Rustype

Code is safe when values and operations remain within the compiler's understood type model and no unchecked dynamic behavior crosses into trusted code.

Safe code should provide strong guarantees around:

- explicit optionality
- explicit recoverable errors
- exhaustive handling of closed variants
- trait satisfaction
- newtype separation
- checked narrowing and conversions
- typed public APIs
- visible state transitions

## Unsafe Rustype

`unsafe` marks code where the compiler cannot prove the normal Rustype guarantees.

```python
unsafe:
    module = importlib.import_module(name)
    handler = getattr(module, handler_name)
```

Typical unsafe operations may include:

- reflection with statically unknown members
- dynamic imports with unknown exports
- monkey patching
- unchecked casts
- calls into untyped Python dependencies
- runtime mutation of class APIs
- foreign objects whose structure cannot be verified
- deliberate bypasses of Rustype's normal type restrictions

## Unsafe is not memory unsafe

Rustype's `unsafe` must never imply raw-pointer or memory semantics. It means only that normal Rustype static guarantees are suspended or weakened within a clearly marked region.

## Safe abstractions over unsafe internals

Rustype should explicitly encourage this Rust pattern:

```python
fn load_plugin(name: str) -> Result[Plugin, PluginError]:
    unsafe:
        module = importlib.import_module(name)
        raw = getattr(module, "plugin")

    return validate_plugin(raw)
```

Callers interact only with the safe return type. Dynamic implementation details remain locally contained.

## Unsafe value escape

A central compiler rule should prevent unknown/unsafe values from silently escaping an unsafe region into safe code.

Values produced in an unsafe region should require one of:

- a proven static type,
- runtime validation,
- an explicit checked conversion,
- or an explicitly unsafe outward API.

## Suppressions

Rustype should minimize generic ignore comments. When suppressions are necessary, they should be specific, machine-readable, and attributable to a diagnostic code.

Potential future syntax:

```text
# rustype: ignore[RT204] -- upstream stub is incorrect
```

The toolchain may later expose suppression/unsafe debt reporting.

## Profiles

The first compiler should have one well-defined language contract rather than a large collection of loosely related lint modes. Optional lint policy can evolve later, but core language-safety rules should not depend on user preference.

## Validation boundaries

Runtime validation frameworks such as Pydantic may become first-class interop boundaries. Rustype should recognize the conceptual transformation:

```text
unknown external value -> validated value -> trusted Rustype type
```

No single validation library should be a hard dependency of the language.
