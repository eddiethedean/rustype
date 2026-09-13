# Rustype v0 Grammar

Status: **Draft normative grammar**

This grammar defines the syntactic shape of Rustype v0. It is written in an EBNF-like notation for design and parser implementation planning. Lexical details intentionally follow Python where not overridden here.

The companion normative syntax rules are in `V0_SYNTAX_SPEC.md`.

## Notation

```text
rule        ::= production
A B         sequence
A | B       alternatives
[A]         optional
{A}         zero or more
A+          one or more
"text"      literal token
NAME        lexical token / referenced grammar rule
```

Indentation tokens are modeled as `NEWLINE`, `INDENT`, and `DEDENT` as in Python-style parsers.

## Module

```text
module ::= { NEWLINE | statement } EOF
```

## Statements

```text
statement ::= compound_statement
            | simple_statement NEWLINE

simple_statement ::= import_statement
                   | assignment_statement
                   | augmented_assignment
                   | return_statement
                   | raise_statement
                   | pass_statement
                   | break_statement
                   | continue_statement
                   | expression_statement

compound_statement ::= function_declaration
                     | class_declaration
                     | enum_declaration
                     | trait_declaration
                     | newtype_declaration
                     | if_statement
                     | if_let_statement
                     | while_statement
                     | for_statement
                     | match_statement
                     | try_statement
                     | with_statement
                     | unsafe_statement
                     | decorated_declaration
```

## Imports

```text
import_statement ::= "import" dotted_as_names
                   | "from" dotted_name "import" import_as_names

dotted_as_names ::= dotted_as_name { "," dotted_as_name }
dotted_as_name  ::= dotted_name [ "as" NAME ]

import_as_names ::= import_as_name { "," import_as_name }
import_as_name  ::= NAME [ "as" NAME ]

dotted_name ::= NAME { "." NAME }
```

`from x import *` is not in the v0 grammar.

## Decorators

```text
decorated_declaration ::= decorator+ (function_declaration | class_declaration)

decorator ::= "@" expression NEWLINE
```

## Functions

```text
function_declaration ::= [ "async" ] "fn" NAME [ type_parameters ]
                         "(" [ parameters ] ")"
                         "->" type_expression ":" suite

parameters ::= parameter { "," parameter } [ "," ]

parameter ::= normal_parameter
            | variadic_parameter
            | keyword_variadic_parameter
            | keyword_only_marker

normal_parameter ::= NAME ":" type_expression [ "=" expression ]
                   | ("self" | "cls") [ ":" type_expression ]

variadic_parameter ::= "*" NAME ":" type_expression
keyword_variadic_parameter ::= "**" NAME ":" type_expression
keyword_only_marker ::= "*"
```

Semantic ordering constraints for required/defaulted/variadic parameters follow Python call semantics.

## Type parameters

```text
type_parameters ::= "[" type_parameter { "," type_parameter } [ "," ] "]"

type_parameter ::= NAME [ ":" type_expression ]
```

v0 allows a single bound per type parameter.

## Classes

```text
class_declaration ::= "class" NAME [ type_parameters ]
                      [ "(" [ argument_list ] ")" ]
                      ":" suite
```

Class bodies may contain assignments, methods, nested supported declarations, and ordinary v0 statements.

## Newtypes

```text
newtype_declaration ::= "newtype" NAME "=" type_expression NEWLINE
```

## Enums

```text
enum_declaration ::= "enum" NAME [ type_parameters ] ":" NEWLINE INDENT
                     enum_variant+
                     DEDENT

enum_variant ::= NAME NEWLINE
               | NAME "(" [ enum_fields ] ")" NEWLINE

enum_fields ::= enum_field { "," enum_field } [ "," ]
enum_field  ::= NAME ":" type_expression
```

All enum payload fields are named in v0.

## Traits

```text
trait_declaration ::= "trait" NAME [ type_parameters ] ":" NEWLINE INDENT
                      trait_method+
                      DEDENT

trait_method ::= "fn" NAME [ type_parameters ]
                 "(" [ parameters ] ")"
                 "->" type_expression NEWLINE
```

Trait methods have signatures only in v0.

## Assignment

```text
assignment_statement ::= assignment_target [ ":" type_expression ] "=" expression
                       | target_list "=" expression

augmented_assignment ::= assignment_target aug_op expression

aug_op ::= "+=" | "-=" | "*=" | "/=" | "//=" | "%="
         | "**=" | "&=" | "|=" | "^=" | ">>=" | "<<="
```

Walrus assignment (`:=`) is excluded.

## Return and raise

```text
return_statement ::= "return" [ expression ]
raise_statement  ::= "raise" [ expression [ "from" expression ] ]
```

## If

```text
if_statement ::= "if" expression ":" suite
                 { "elif" expression ":" suite }
                 [ "else" ":" suite ]
```

## If-let

```text
if_let_statement ::= "if" "let" pattern "=" expression ":" suite
                     [ "else" ":" suite ]
```

`let` is only valid as part of `if let` in v0; standalone `let` statements are reserved and rejected.

## While

```text
while_statement ::= "while" expression ":" suite
                    [ "else" ":" suite ]
```

`while let` is not part of v0.

## For

```text
for_statement ::= [ "async" ] "for" target_list "in" expression ":" suite
                  [ "else" ":" suite ]
```

## Match

```text
match_statement ::= "match" expression ":" NEWLINE INDENT
                    match_arm+
                    DEDENT

match_arm ::= pattern [ "if" expression ] ":" suite
```

Rustype deliberately omits Python's `case` keyword in v0 arms. The indented pattern itself begins each arm.

## Patterns

```text
pattern ::= wildcard_pattern
          | literal_pattern
          | capture_pattern
          | variant_pattern

wildcard_pattern ::= "_"
literal_pattern  ::= literal
capture_pattern  ::= NAME

variant_pattern ::= qualified_name
                  | qualified_name "(" [ pattern_arguments ] ")"

pattern_arguments ::= pattern { "," pattern } [ "," ]

qualified_name ::= NAME { "." NAME }
```

Semantic analysis resolves whether a qualified name is a Rustype enum variant, `Some`, `None`, `Ok`, or `Err`.

## Try

```text
try_statement ::= "try" ":" suite except_clause+
                  [ "else" ":" suite ]
                  [ "finally" ":" suite ]
                | "try" ":" suite "finally" ":" suite

except_clause ::= "except" expression [ "as" NAME ] ":" suite
```

Bare `except:` is intentionally absent.

## With

```text
with_statement ::= [ "async" ] "with" with_items ":" suite

with_items ::= with_item { "," with_item }
with_item  ::= expression [ "as" assignment_target ]
```

## Unsafe

```text
unsafe_statement ::= "unsafe" ":" suite
```

`unsafe` is a Rustype static-safety boundary and has no memory-management semantics.

## Suite

```text
suite ::= simple_statement NEWLINE
        | NEWLINE INDENT statement+ DEDENT
```

## Expressions

The expression grammar follows Python precedence with the addition of postfix `?`.

```text
expression ::= lambda_expression
             | conditional_expression

lambda_expression ::= "lambda" [ lambda_parameters ] ":" expression

conditional_expression ::= or_expression
                           [ "if" or_expression "else" expression ]

or_expression ::= and_expression { "or" and_expression }
and_expression ::= not_expression { "and" not_expression }
not_expression ::= "not" not_expression | comparison

comparison ::= bitwise_or { comparison_op bitwise_or }

comparison_op ::= "<" | ">" | "==" | ">=" | "<=" | "!="
                | "in" | "not" "in" | "is" | "is" "not"

bitwise_or  ::= bitwise_xor { "|" bitwise_xor }
bitwise_xor ::= bitwise_and { "^" bitwise_and }
bitwise_and ::= shift_expr { "&" shift_expr }
shift_expr  ::= sum { ("<<" | ">>") sum }
sum         ::= term { ("+" | "-") term }
term        ::= factor { ("*" | "@" | "/" | "//" | "%") factor }
factor      ::= ("+" | "-" | "~") factor | power
power       ::= await_primary [ "**" factor ]

await_primary ::= [ "await" ] primary
```

## Primary and postfix `?`

```text
primary ::= atom postfix_op*

postfix_op ::= call_suffix
             | attribute_suffix
             | subscript_suffix
             | question_suffix

call_suffix      ::= "(" [ argument_list ] ")"
attribute_suffix ::= "." NAME
subscript_suffix ::= "[" slice_list "]"
question_suffix  ::= "?"
```

Postfix operations associate left-to-right. Therefore:

```text
parse(text)?.normalize()
```

is parsed as:

```text
((parse(text))?).normalize()
```

Semantic analysis restricts `?` to supported Rustype types and compatible containing functions.

## Atoms

```text
atom ::= NAME
       | literal
       | list_display
       | tuple_or_group
       | dict_or_set_display
       | comprehension
       | f_string
```

The exact lexical grammar for Python-compatible literals and displays may be reused from a Python parser implementation where possible.

## Arguments

```text
argument_list ::= argument { "," argument } [ "," ]

argument ::= expression
           | NAME "=" expression
           | "*" expression
           | "**" expression
```

## Type expressions

Rustype v0 type expressions are intentionally constrained.

```text
type_expression ::= type_union

type_union ::= type_primary { "|" type_primary }

type_primary ::= qualified_name
               | qualified_name "[" type_arguments "]"
               | "None"

type_arguments ::= type_expression { "," type_expression } [ "," ]
```

Examples:

```text
int
User
list[User]
Option[User]
Result[User, UserError]
User | None
```

Arbitrary calls, arithmetic, lambdas, and runtime-computed values are not legal declared Rustype type expressions in v0.

## Literals

```text
literal ::= INTEGER
          | FLOAT
          | COMPLEX
          | STRING
          | BYTES
          | "True"
          | "False"
          | "None"
```

Collection literals are atoms rather than scalar literal tokens.

## Assignment targets

```text
assignment_target ::= NAME
                    | primary "." NAME
                    | primary "[" slice_list "]"
                    | target_list

target_list ::= assignment_target "," assignment_target { "," assignment_target }
```

Semantic restrictions may reject targets that are syntactically valid but unsafe under the active Rustype profile.

## Excluded syntax in v0

The parser must reject these rather than silently treating them as future-compatible:

```text
def / async def
:=
yield
yield from
while let
standalone let
let mut
impl
where clauses
macro syntax
Rust references (&T, &mut T)
lifetime syntax
Rust move syntax
```

## Parser implementation note

The recommended implementation strategy is not to fork all of Python's grammar manually. Instead:

1. Reuse Python lexical conventions where possible.
2. Implement a Rustype parser with explicit grammar productions for Rustype declarations and postfix `?`.
3. Reuse or port stable Python expression productions where they preserve Rustype semantics.
4. Keep source spans on every token and AST node from the first prototype.

The grammar is expected to evolve during v0 implementation, but changes should update both this document and `V0_SYNTAX_SPEC.md` in the same commit.