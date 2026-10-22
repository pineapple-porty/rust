# Urk source implementation rules

This document is the source-language implementation specification. Treat the
language rules below as normative. Do not add behavior that is not specified.

## Rules for implementation

1. Implement Urk source handling in Rust, following the existing crate and
   repository structure. Keep changes focused on the requested language
   feature, and update relevant tests and documentation with the implementation.
2. Keep source processing separate from bytecode execution: source code is
   translated before it reaches the runtime, and the runtime executes bytecode,
   not Urk source. Follow the version 1 implementation contract in
   [runtime.md](./runtime.md), which represents the source types defined here.
3. Preserve the meaning of every syntax rule and example in this document.
   Do not silently normalize, reinterpret, or reject syntax based on unstated
   assumptions.
4. If behavior required for implementation is missing from this document,
   pause and ask the project author rather than inventing a language rule.
5. Keep implementation decisions explicit and testable. Tests should cover
   specified behavior; do not use tests to establish new language semantics
   without the author's agreement.

## Specified language model

Urk is statically typed: the compiler must determine and check the type of
each expression and value before execution. A program with incompatible types
must be rejected during compilation, not deferred to runtime. Function
parameters, local containers, and the result type of every non-entry function
that returns a value require explicit annotations. Integer literals use their
surrounding expected type when available; otherwise they require a type suffix.

Urk uses **center syntax** to express an operation. A condition or guard comes
first, the value is in the center, and an operator, function, or other action
follows it:

```text
<condition><value><operator, function, etc.>
```

Source files are UTF-8. Keywords and type names are case-sensitive.
Horizontal whitespace is a space or tab. LF and CRLF are accepted as
newlines. Newlines and semicolons separate statements at the top level and
inside function blocks. Newlines inside parentheses are whitespace;
semicolons are not allowed inside parentheses. Statements may continue
across lines inside parentheses, but not otherwise. Every operation uses the
grammar below.

### Template objects and ownership

Definitions use template objects. A template is a copyable object that gives
data an owner until the data is passed into a real variable or a function.
The `container` template declares a named, typed local slot; the quoted name
is the slot's name, not a string value stored in it. A value assigned to a
container must have exactly its declared type. A container can be assigned
again, but its type cannot change. Integer and `u1` values are copied when
assigned, passed to functions, or returned. String values are immutable
references to static literal data and are copied by reference. Unused
temporary values are discarded at the end of their containing expression.
`functiontemp makes "name"` declares a non-entry function symbol named
`name`; its signature and definition use that same name. It is required once
for every non-entry function. The entry function `main` is implicit and must
not be declared with `functiontemp`.

Parameters and containers are local to their function. A container is visible
from its declaration to the end of its function body. Duplicate names in one
function are errors; shadowing is not allowed. Function names are module-wide
and unique. Function signatures must match the parameters in their definition.
Containers start uninitialized. The compiler performs definite-assignment
analysis and rejects every path that can read a container before assigning
it.
Names stored in template declarations and function signatures must decode to
valid, non-reserved Urk identifiers. Function names and local names occupy
separate namespaces.

### Types and literals

The code blocks in this document are independent examples unless a section
explicitly says they are part of the same program.

The supported integer types are Rust's signed and unsigned integer types:
`i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`,
`u128`, and `usize`, plus the Urk-specific `u1` type. `u1` is an unsigned
one-bit integer whose only values are `0` and `1`; it is the type for truth
and condition values.

For an `iN` type, the range is `-2^(N-1)` through `2^(N-1)-1`; for a `uN`
type, the range is `0` through `2^N-1`. `isize` and `usize` use the target
pointer width, which must be 32 or 64 bits.

`str` is a UTF-8 string type whose values refer to immutable literal bytes
stored in the compiled program. String literals may be assigned to `str`
containers, passed to functions, and returned from functions. String contents
cannot be mutated or concatenated. Dynamically allocated heap strings are
deferred.

Integer literals are decimal digits, optionally followed immediately by an
integer type suffix such as `42i32` or `1u1`. An unsuffixed integer literal
must have an expected type from its declaration, assignment, function
argument, return context, or the other operand of an operation; otherwise
compilation fails. No implicit conversions are performed. Integer arithmetic
is checked: overflow, division by zero, and signed minimum divided by negative
one are execution errors. Signed remainder follows Rust's remainder rule;
signed minimum remainder negative one is also an overflow error.

Comparisons between operands of the same integer type produce `u1`. Conditions
must have type `u1`; `1` means true and `0` means false. The `u1` type is not
used for arithmetic. Integer operands of different types do not implicitly
coerce.

The typed container declaration form is:

```text
container as i32 makes "name"
```

The example below declares a container named `y`, then passes `12` to
`my_function` using that name:

```text
functiontemp makes "my_function"
"my_function" uses (arg as i32)
my_function for {}
main for {
container as i32 makes "y"
when 12 uses "y" for my_function
}
```

In this example, `uses` assigns the value to the named owner and `for` passes
the value into a function. A typed container must declare its type when it is
created. In this single-argument call form, the value is assigned to the
named container and that container is passed as the function's sole argument.
This shorthand is valid only for a function with no result.

Functions with multiple arguments are called by placing their ordered
argument list at the center:

```text
functiontemp makes "guessing_game"
"guessing_game" uses (num1 as i32, num2 as i32, guess as i32)
guessing_game return u1 for {
    if num1 * num2 == guess 1 uses u1 for return
    0 uses u1 for return
}
main for {
    container as u1 makes "answer"
    (12, 3, 36) uses "answer" for guessing_game
}
```

Literal argument types come from the called function's signature. A
result-bearing call must name its destination container, and that container
must have the function's result type. A no-result call must omit the
destination. Calls are statements, not expressions; their result is never
implicitly discarded and cannot be nested inside another expression.

### Function example

The following example describes a function named `guessing_game` with three
`i32` parameters. Its intended result is `1` if `num1 * num2` equals `guess`,
and `0` otherwise:

```text
when functiontemp makes "guessing_game"
"guessing_game" uses (num1 as i32, num2 as i32, guess as i32)
guessing_game return u1 for {
    if num1 * num2 == guess 1 uses u1 for return
    0 uses u1 for return
}
```

This illustrates typed parameters and a typed function result. Function
signatures precede their definitions. `if` executes its following return
operation only when its condition is `1`; each reachable path in a function
with a result must return a value of that declared type.

## Source grammar and semantics

The lexical, grammar, expression, scope, and error rules below are normative.
The bytecode target is described in [runtime.md](./runtime.md).

### Lexical rules

The following spellings are reserved words:
`when`, `if`, `makes`, `uses`, `for`, `return`, `as`, all integer type names,
and `str`. The built-in names `container` and `functiontemp` are ordinary
identifiers. `main` is reserved as the program entry-function name.

```ebnf
identifier      = identifier_start, { identifier_continue } ;
identifier_start = letter | "_" ;
identifier_continue = letter | digit | "_" ;

number          = digit, { digit }, [ integer_type_suffix ] ;
integer_type_suffix = integer_type ;
string          = '"', { string_char | escape }, '"' ;
string_char     = ? any Unicode scalar except quotation mark, reverse solidus,
                    or newline ? ;
escape          = ? reverse solidus followed by quotation mark, reverse
                    solidus, "n", "r", or "t" ? ;

letter          = "A"…"Z" | "a"…"z" ;
digit           = "0"…"9" ;
```

Horizontal whitespace separates tokens and is otherwise ignored. A newline
or semicolon separates statements at the top level and inside function blocks.
Inside parentheses, a newline is whitespace and a semicolon is invalid.
`#` begins a comment that runs to the end of the line. Multi-character
operators (`==`, `!=`, `<=`, `>=`) are recognized before their
single-character prefixes. The complete operator set is `+`, `-`, `*`, `/`,
`%`, `==`, `!=`, `<`, `<=`, `>`, and `>=`; lone `=` or `!` is invalid.
Identifiers use ASCII letters and underscores as their first character and
ASCII letters, digits, or underscores thereafter.
Quoted symbol names must decode to a valid, non-reserved identifier. Integer
literals use decimal digits; leading zeroes are allowed. A type suffix, when
present, is the immediately following integer type name. Strings use UTF-8
source text and support only the escapes listed above. The escapes `\"`,
`\\`, `\n`, `\r`, and `\t` represent quotation mark, reverse solidus, line
feed, carriage return, and tab respectively. Unknown or incomplete escapes
are lexical errors. A numeric token followed immediately by identifier
characters must have a recognized integer type suffix and no trailing
identifier characters; otherwise it is a lexical error.

### Grammar

```ebnf
program             = { separator }, [ module_statement,
                      { separator, module_statement }, { separator } ], EOF ;
separator           = newline | ";" ;

module_statement    = template_declaration
                    | function_signature
                    | function_definition ;
block_statement     = declaration | conditional_return | value_operation
                    | function_call ;

declaration         = [ "when" ], "container", "as", type, "makes", string ;
template_declaration = [ "when" ], "functiontemp", "makes", string ;
function_signature  = string, "uses", "(", [ parameter_list ], ")" ;
parameter_list      = parameter, { ",", parameter } ;
parameter           = identifier, "as", type ;

function_definition = identifier, [ "return", type ], "for", block ;
block               = "{", { separator }, [ block_statement,
                      { separator, block_statement }, { separator } ], "}" ;

conditional_return  = "if", expression, value, "uses", type,
                      "for", "return" ;
value_operation     = [ "when" ], value, operation ;
function_call       = "(", [ argument_list ], ")", "uses", [ string ],
                      "for", identifier ;
argument_list       = value, { ",", value } ;
operation           = "uses", string, [ "for", identifier ]
                    | "uses", type, "for", "return"
                    | "makes", string ;

type                = "u1" | "i8" | "i16" | "i32" | "i64" | "i128"
                    | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
                    | "usize" | "str" ;
integer_type        = "u1" | "i8" | "i16" | "i32" | "i64" | "i128"
                    | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
                    | "usize" ;
value               = expression | string ;
expression          = comparison ;
comparison          = additive, [ comparison_operator, additive ] ;
comparison_operator = "==" | "!=" | "<" | "<=" | ">" | ">=" ;
additive            = multiplicative, { ( "+" | "-" ), multiplicative } ;
multiplicative      = unary, { ( "*" | "/" | "%" ), unary } ;
unary               = [ "-" ], primary ;
primary             = number | identifier | "(", expression, ")" ;
```

Every program must define exactly one `main for { ... }` entry function. It
takes no parameters and has no return value. Other functions that return a
value must declare its type with `return <type>`; a function without that
clause returns no value. Return operations inside a function must match its
declared result type.
Each function call supplies exactly the declared number of arguments, in
parameter order, with exactly matching types. A single value may be assigned
to a container and passed as one argument using `uses "<container>" for
<function>`; an argument list calls a function with multiple values. Arguments
are evaluated from left to right. Function overloading is not supported.
Every non-entry function requires one signature declaration. Signatures must
appear before their function definitions; every called function must have a
signature. The quoted signature name must match the definition's identifier.
The compiler assigns bytecode function IDs starting at zero in source
definition order, including `main`; these IDs do not appear in Urk source.
`main` has no function-template declaration or separate signature and cannot
be called as a normal function. It cannot return a value. Functions without a
result cannot contain a return-value operation. Recursive calls are allowed.
A source function with no result compiles to a bytecode function with no
result; `void` is not a source type. Reaching the end of a no-result function
returns normally; the compiler emits the corresponding bytecode `RETURN`.

### Precedence and interpretation

1. Parentheses group expressions.
2. Unary `-` binds most tightly and applies only to signed integer types.
3. `*`, `/`, and `%` bind more tightly than `+` and `-`.
4. Comparisons bind less tightly than arithmetic; chained comparisons are
   rejected. Both operands must have the same integer type.
5. Arithmetic operators associate left to right and require operands of the
   same integer type. They do not accept `u1` or `str`.
6. `when` is an optional no-error guard marker; it does not produce a value
   or alter control flow. It does not catch or suppress errors.
   `if <expression>` introduces a conditional return; its expression must
   have type `u1`, and the return occurs only when its value is `1`.
7. `uses` binds the preceding value to the following owner. In
   `<value> uses "<container>" for <function>`, the value is assigned to the
   container and that container is passed as the function's sole argument.
   In `(<arguments>) uses ["<destination>"] for <function>`, arguments are
   passed in order; a result-bearing function requires the destination, which
   receives its returned value. A no-result function omits the destination.
   `makes` declares the quoted name from the preceding template.
8. A function signature is a separate statement associating a quoted function
   name with typed parameters. Its definition uses that name without quotes.
9. String values have type `str` and are immutable literal-backed values.
    Strings do not support comparison, mutation, or concatenation.
10. No implicit integer conversions, logical operators, loops, or `else`
    branches are part of the language.
11. The type following `uses` in a return operation must exactly match the
    enclosing function's declared result type; it does not convert the value.
12. The compiler lowers unary negation to checked subtraction from zero of
    the operand's type.

An unsuffixed integer literal takes an expected type from its declaration,
assignment, function argument, return, or the other operand of a typed
operation. If no expected type is available, compilation fails and the
literal must have a suffix. The literal must fit its selected type; a negative
sign is applied after parsing the positive magnitude, with the signed minimum
value specially permitted. An out-of-range literal is a compile-time error.
Arithmetic overflow and division errors are execution traps, even when the
compiler can evaluate the expression at compile time.

Compilation errors must identify the source file and line and column where
the error was detected. Line and column numbers are one-based; columns count
Unicode scalar values, with a tab counting as one column. The compiler may
stop after the first error; error recovery is not required.

### Author decisions before implementation

The rules above are the current implementation target. Confirm these remaining
choices before treating the source-language semantics as closed:

- Does the expected type of an assignment, argument, or return propagate
  through an entire arithmetic expression containing only unsuffixed literals
  (for example, `container as i32 makes "x"` followed by `1 + 2 uses "x"`),
  or must at least one operand establish the type independently?
- How should the signed minimum literal be written and tokenized? The text
  permits the minimum value specially after applying unary minus, but does not
  explicitly settle forms such as `-128i8` versus `-(128i8)`.
- Must each `functiontemp` declaration and function signature correspond to
  exactly one definition, and what ordering is required between the template
  declaration and signature?
- The grammar allows `when` only before declarations, template declarations,
  and value operations, not calls or conditional returns. Is that scope
  intentional, and is `when` permanently a semantic no-op? The current rule
  specifies no effect and suppresses no errors.
- The grammar permits a value followed by `makes "<name>"`, while the examples
  use `makes` to declare a name from a template. Which values/templates may
  legally use this form, and what does it create?
- Are orphan function-template declarations, orphan signatures, duplicate
  signatures, and duplicate template declarations all compile-time errors?
