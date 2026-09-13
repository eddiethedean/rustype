# Rustype v0 Syntax Specification

Status: **Draft v0 language contract**

This document defines the concrete syntax accepted by the first Rustype compiler prototype. It is intentionally smaller than the long-term language vision.

Rustype source files use the `.rpy` extension and compile to Python.

## 1. Design rules

Rustype v0 follows five rules:

1. Python remains the baseline syntax and runtime model.
2. Rust-inspired syntax is added only where it expresses a stronger static guarantee.
3. Rustype does not implement ownership, borrowing, lifetimes, moves, pointer semantics, or memory management.
4. All accepted Rustype code must lower deterministically to ordinary Python.
5. v0 prefers a small coherent language over broad Python syntax parity.

## 2. File structure

A `.rpy` file is a module.

A module contains zero or more statements and declarations.

Supported top-level forms in v0:

- imports
- constant/value assignments
- `newtype`
- `enum`
- `trait`
- `class`
- `fn`
- `async fn`
- decorated `class`, `fn`, and `async fn`

Example:

```python
from pydantic import BaseModel

newtype UserId = int

enum LookupError:
    NotFound(id: UserId)
    Backend(message: str)

trait Repository:
    fn get(self, id: UserId) -> Result[User, LookupError]

class User(BaseModel):
    id: UserId
    name: str

fn load_user(repo: Repository, id: UserId) -> Result[User, LookupError]:
    return repo.get(id)
```

## 3. Significant indentation

Rustype uses Python-style significant indentation.

- A block begins after `:` followed by a newline.
- Nested blocks must be indented consistently.
- Tabs are rejected in v0 source.
- The compiler recommends four spaces, but exact width is not semantically significant as long as indentation levels are consistent.

## 4. Comments

Line comments use `#`.

```python
# This is a comment.
fn add(a: int, b: int) -> int:
    return a + b
```

Multiline comments do not exist. Triple-quoted strings are strings, not comments.

## 5. Identifiers

Rustype uses Python-compatible identifiers.

Examples:

```text
user
User
_user
user_id
Repository2
```

Reserved Rustype keywords in v0:

```text
fn
trait
enum
newtype
unsafe
let
mut
```

`let` and `mut` are reserved for future use but are not valid statements in v0.

Rustype also reserves Python keywords.

## 6. Literals

v0 accepts Python-compatible literals:

- integers
- floats
- complex numbers
- strings
- bytes
- booleans
- `None`
- list literals
- tuple literals
- dict literals
- set literals

Examples:

```python
count = 3
name = "Eddie"
active = True
items = [1, 2, 3]
point = (10, 20)
metadata = {"source": "api"}
```

Rustype's `Option.None` variant is represented in source by `None` when the expected type is `Option[T]`. The semantic analyzer distinguishes ordinary Python `None` from the `Option` empty variant by context.

## 7. Type syntax

v0 adopts Python type-expression syntax.

Supported examples:

```python
int
str
User
list[User]
dict[str, int]
tuple[int, str]
User | None
Option[User]
Result[User, LookupError]
Repository
T
```

Rustype-specific type constructors in v0:

```text
Option[T]
Result[T, E]
Never
```

Union syntax uses `|`.

## 8. Imports

Python import syntax is preserved.

```python
import json
import polars as pl
from pydantic import BaseModel
from app.models import User
```

Star imports are rejected in v0:

```python
from package import *  # compile error
```

## 9. Bindings and assignment

v0 preserves Python assignment syntax.

```python
name = "Ada"
count: int = 0
count += 1
```

Multiple assignment is accepted:

```python
x, y = point
```

Walrus expressions (`:=`) are excluded from v0.

`let` and `let mut` are reserved for future investigation but intentionally absent from v0.

## 10. Functions

Rustype-defined functions use `fn` instead of `def`.

```python
fn add(a: int, b: int) -> int:
    return a + b
```

Async functions use:

```python
async fn fetch_user(id: UserId) -> Result[User, FetchError]:
    ...
```

### 10.1 Parameter syntax

v0 supports:

```python
fn example(
    required: int,
    defaulted: str = "x",
    *args: str,
    flag: bool,
    **kwargs: object,
) -> None:
    ...
```

Parameter annotations are required for Rustype functions except `self` and `cls`.

Return annotations are required for every Rustype function.

### 10.2 `def`

Plain Python `def` and `async def` are **not valid Rustype declarations in v0**.

This keeps the language boundary explicit. Python functions remain accessible by importing `.py` modules.

### 10.3 Decorators

Decorators use Python syntax:

```python
@app.get("/users/{id}")
async fn get_user(id: UserId) -> Result[User, UserError]:
    ...
```

Decorators must be valid expressions.

## 11. Return

Python `return` syntax is preserved.

```python
return user
return Ok(user)
return Err(NotFound(id))
```

A function declared `-> Result[T, E]` must return a compatible `Result` on every reachable return path.

## 12. `Option`

`Option[T]` is a closed Rustype type with two variants:

```text
Some(T)
None
```

Construction:

```python
return Some(user)
return None
```

`Option[T]` cannot be implicitly converted to `T`.

Supported v0 methods:

```text
is_some()
is_none()
unwrap()
expect(message)
unwrap_or(default)
map(fn)
and_then(fn)
```

Bare `unwrap()` is accepted in v0 but may produce a lint-level diagnostic.

## 13. `Result`

`Result[T, E]` has two variants:

```text
Ok(T)
Err(E)
```

Construction:

```python
return Ok(user)
return Err(NotFound(id))
```

Supported v0 methods:

```text
is_ok()
is_err()
unwrap()
expect(message)
unwrap_or(default)
map(fn)
map_err(fn)
and_then(fn)
```

## 14. Postfix `?`

Postfix `?` is a Rustype expression operator.

```python
user = repo.get(id)?
```

v0 semantics:

- Operand must be `Result[T, E1]` or `Option[T]`.
- For `Result`, the containing function must return `Result[U, E2]` where the error is compatible with `E2`.
- `Ok(value)?` evaluates to `value`.
- `Err(error)?` returns the compatible error from the containing function.
- For `Option`, `Some(value)?` evaluates to `value` and `None?` returns `None` from an `Option[...]`-returning function.
- `?` is rejected outside a function body.
- `?` is rejected in a function with an incompatible return type.

Precedence: postfix `?` binds with other postfix operations and more tightly than binary operators.

```python
value = parse(text)?.normalize()
```

is parsed as:

```text
(parse(text)?).normalize()
```

## 15. Newtypes

Syntax:

```python
newtype UserId = int
newtype Email = str
```

v0 rules:

- Right-hand side must be a valid type expression.
- A newtype is statically distinct from its wrapped type and other newtypes.
- Construction uses call syntax:

```python
id = UserId(42)
```

- Explicit extraction uses `.value` in v0:

```python
raw: int = id.value
```

Runtime lowering may use a lightweight generated wrapper or compiler runtime helper. Runtime representation is an implementation detail as long as Python interoperability remains predictable.

## 16. Enums

Rustype enums are closed sum types.

Syntax:

```python
enum Message:
    Quit
    Move(x: int, y: int)
    Write(value: str)
```

Variant forms:

```text
UnitVariant
TupleLikeVariant(name: Type, ...)
```

All payload fields are named in v0.

Variant construction:

```python
Quit
Move(x=1, y=2)
Write(value="hello")
```

For ergonomic construction, positional arguments matching declaration order are also accepted:

```python
Move(1, 2)
```

Variant names share the enum's module namespace in v0.

## 17. Pattern matching

Rustype uses Python's `match` keyword with Rustype-specific arm syntax.

```python
match message:
    Quit:
        stop()
    Move(x, y):
        move_to(x, y)
    Write(value):
        print(value)
```

Supported v0 patterns:

- enum unit variant
- enum payload variant
- `Some(name)`
- `None`
- `Ok(name)`
- `Err(name)`
- literal patterns
- wildcard `_`
- simple capture names

Nested algebraic patterns are supported:

```python
match result:
    Ok(Some(user)):
        use(user)
    Ok(None):
        missing()
    Err(error):
        fail(error)
```

Guards use Python syntax:

```python
match value:
    Some(x) if x > 0:
        ...
    _:
        ...
```

### 17.1 Exhaustiveness

A match over a Rustype closed type (`enum`, `Option`, `Result`) must be exhaustive.

A wildcard `_` counts as exhaustive.

Unreachable duplicate arms are compile-time errors when statically detectable.

## 18. `if let`

Syntax:

```python
if let Some(user) = find_user(id):
    print(user.name)
```

Optional `else` is supported:

```python
if let Ok(value) = parse(text):
    use(value)
else:
    handle_failure()
```

v0 does not support `while let`.

## 19. Traits

Syntax:

```python
trait Repository:
    fn get(self, id: UserId) -> Option[User]
    fn save(self, user: User) -> Result[None, SaveError]
```

v0 trait rules:

- Trait bodies contain function signatures only.
- Trait methods have no body.
- `self` and `cls` need no explicit type annotation.
- All other parameters must be annotated.
- Return annotations are required.
- Traits are structurally satisfied by default.
- Explicit `impl` blocks are not part of v0.

A normal class satisfies a trait if its public method signatures are compatible.

## 20. Generics and trait bounds

v0 uses Python 3.12-style type parameter syntax with Rustype bounds.

```python
fn load[R: Repository](repo: R, id: UserId) -> Option[User]:
    return repo.get(id)
```

Generic classes use:

```python
class Box[T]:
    value: T
```

Multiple type parameters are allowed:

```python
class Pair[A, B]:
    first: A
    second: B
```

v0 supports a single bound per type parameter. `where` clauses are deferred.

## 21. Classes

Normal Python-style classes are supported:

```python
class User:
    id: UserId
    name: str

    fn display_name(self) -> str:
        return self.name
```

Methods use `fn`, not `def`.

Inheritance is supported:

```python
class Admin(User):
    level: int
```

Multiple inheritance is accepted syntactically in v0 but may be restricted later if it prevents sound static analysis.

Metaclass syntax is excluded from v0.

## 22. `unsafe`

Syntax:

```python
unsafe:
    module = importlib.import_module(name)
    handler = getattr(module, "handler")
```

`unsafe` marks a static-safety escape region. It has no memory-safety meaning.

Operations expected to require `unsafe` in the strict v0 checker include:

- explicit `Any`
- calls into known-untyped Python APIs
- dynamic import helpers
- unchecked `getattr` / `setattr`
- monkey patching
- reflection that bypasses statically known members
- unchecked casts

The exact unsafe-operation catalog may grow, but unsafe blocks must remain lexically explicit.

Values created inside `unsafe` are not automatically trusted outside it. The semantic analyzer may require explicit validation, narrowing, or a declared typed boundary before such values can flow into safe code.

## 23. Exceptions

Python exception syntax remains supported:

```python
try:
    value = int(text)
except ValueError as error:
    return Err(ParseError(str(error)))
```

Rustype encourages `Result` for recoverable domain failures, but exceptions remain necessary for Python interoperability.

Supported v0 forms:

- `raise`
- `try`
- `except`
- `else`
- `finally`

Bare `except:` is rejected.

## 24. Control flow

v0 preserves these Python statements:

```text
if / elif / else
for
while
break
continue
pass
return
match
try / except / else / finally
with
async with
async for
```

Comprehensions are supported.

Generator `yield` and `yield from` are deferred from v0.

## 25. Expressions

v0 supports Python-compatible expressions unless explicitly excluded.

Included:

- arithmetic
- comparisons
- boolean operators
- attribute access
- indexing/slicing
- calls
- lambdas
- conditional expressions
- comprehensions
- `await`
- f-strings
- assignment targets
- postfix `?`

Excluded in v0:

- walrus `:=`
- generator expressions using `yield`
- dynamically constructed annotations via arbitrary runtime calls when used as declared Rustype types

## 26. Lambda syntax

Python lambda syntax is retained:

```python
items.map(lambda x: x + 1)
```

Lambda parameters are inferred from context where possible. Explicit lambda parameter annotations are not introduced in v0.

## 27. `with` and context managers

Python syntax is preserved:

```python
with open(path) as file:
    text = file.read()
```

Rustype does not attach ownership or lifetime semantics to context managers.

## 28. Async

Python async syntax is preserved around Rustype functions:

```python
async fn fetch(url: str) -> Result[str, FetchError]:
    response = await client.get(url)
    return Ok(response.text)
```

Supported async constructs:

- `async fn`
- `await`
- `async for`
- `async with`

## 29. Python interoperability boundary

`.rpy` may import `.py` modules directly.

Python APIs with usable annotations can participate in Rustype static analysis.

Untyped or dynamically typed APIs may require an `unsafe` boundary.

Rustype-generated Python must remain importable by ordinary Python.

## 30. Reserved but not implemented in v0

The following Rust-inspired syntax is reserved or explicitly deferred:

```text
let
let mut
impl
where
while let
const generics
associated types
associated constants
macro syntax
? on custom Try-like types
```

They must not be accepted silently by the v0 parser.

## 31. Permanently out of scope under the current mission

Rustype does not implement or emulate Rust's memory model:

```text
ownership
borrowing
& / &mut references
lifetimes
move semantics
Copy / Clone memory semantics
Drop semantics
Box / Rc / Arc memory semantics
pointer aliasing rules
allocator rules
memory layout guarantees
borrow checking
```

If future proposals require these concepts to work correctly, they should be rejected unless Rustype's mission is explicitly redefined.

## 32. v0 conformance rule

A source file conforms to Rustype v0 if:

1. It parses according to `GRAMMAR.md`.
2. It uses only syntax listed as supported in this specification.
3. All Rustype declarations satisfy required annotations and semantic constraints.
4. Every closed-type match is exhaustive.
5. `?` is used only in compatible `Result` or `Option` contexts.
6. Unsafe operations occur only inside an allowed static-safety boundary.
7. The source can be lowered to ordinary Python without changing its defined Rustype behavior.

This document is normative for syntax decisions in the first compiler prototype.