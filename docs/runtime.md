# Urk bytecode runtime

This document defines the version 1 implementation contract for `runtime.exe`.
The contract is designed to be compact, portable, and safely verifiable before
execution. It describes the target architecture; the repository does not yet
contain this runtime or a bytecode compiler. Treat version 1 as the normative
implementation target, but not as a frozen public ABI until the compiler and
runtime implement and test it together.

## Responsibilities and boundaries

The runtime manager translates Urk source into a bytecode module. The runtime
accepts that module, validates it, and executes its entry function. The runtime
must not parse or execute Urk source code. The service center remains the
user-facing entry point described in [layout.md](./layout.md).

The statically typed source language and its supported types are specified in
[syntax.md](./syntax.md). This bytecode contract represents those types:
fixed-width signed and unsigned integers, pointer-width `isize`/`usize`, `u1`,
and immutable UTF-8 `str` literals. A `main` source function compiles to the
required zero-argument, no-result entry function. Runtime bytecode validation
is a safety check and does not replace compile-time source type checking.

## Executable input

The command-line interface is:

```text
runtime.exe [limit-options] <bytecode-file>
```

Require exactly one input path. Limit options must precede the input path.
Each option takes one positive decimal integer no greater than its
corresponding default:

| Option | Default |
|---|---:|
| `--max-module-bytes N` | 67,108,864 |
| `--fuel N` | 10,000,000 |
| `--max-stack-values N` | 65,536 |
| `--max-locals N` | 4,096 |
| `--max-call-depth N` | 1,024 |

Each option may appear at most once. Reject unknown options, missing values,
zero, malformed numbers, and values above the default. Read the input as
binary module data, enforce the module-size limit while reading, validate the
complete module, then execute its declared entry function. Do not try to
interpret invalid bytecode as source code. The runtime manager is responsible
for producing the bytecode file; no source-to-bytecode fallback belongs in the
runtime.

The current instruction set has no host input/output operations. Successful
execution therefore writes no program output to standard output. Report
diagnostics to standard error and return a nonzero process status on usage,
file, validation, resource-limit, or execution errors. Exact exit-code values
are an implementation detail until the executable is implemented.

## Design goals

- Keep the core instruction set and module format small.
- Use explicit versions, fixed-width fields, and little-endian encoding.
- Validate all module structure and control flow before executing any code.
- Reject malformed or unsupported input with an error; never execute a
  partially validated module.
- Use checked bounds and resource limits. Malformed input must not cause a
  panic, out-of-bounds access, or unbounded allocation.
- Keep the format independent of Rust struct layout and host endianness.
- Record target pointer width explicitly; modules that use `isize` or `usize`
  are executable only on a runtime with the same pointer width.

## Module format: URKB version 1

All integer fields are unsigned little-endian unless stated otherwise. Offsets
are byte offsets from the beginning of the module. The module is a 16-byte
header, followed by a section directory, followed by section data.

### Header

| Offset | Size | Field | Version 1 requirement |
|---:|---:|---|---|
| 0 | 4 | Magic | ASCII `URKB` |
| 4 | 2 | Major version | `1` |
| 6 | 2 | Minor version | `0` |
| 8 | 4 | Module flags | `0` |
| 12 | 2 | Section count | `3` |
| 14 | 2 | Reserved | `0` |

Version 1 requires exactly one constants section, one functions section, and
one module section. The directory immediately follows the header. Each
directory entry is 12 bytes:

| Size | Field | Meaning |
|---:|---|---|
| 2 | Section kind | Section type identifier |
| 2 | Section flags | `0` for version 1 sections |
| 4 | Offset | Start of section data in the module |
| 4 | Length | Section length in bytes |

Section kinds are `1` for constants, `2` for functions, and `3` for module
metadata. Directory entries may appear in any order. Section data must begin after the complete directory; sections must be in
bounds and must not overlap. Sections have no alignment requirement, and
padding is not allowed.
The three section ranges, sorted by offset, must cover every byte from the end
of the directory to the end of the module: gaps and unreferenced trailing
bytes are invalid. All offset-plus-length calculations must be checked for
integer overflow. Each section parser must consume its entire section.
Version 1 limits total module size to `u32::MAX` bytes. Validate ranges using
checked arithmetic wide enough to detect overflow before comparing them with
the module length.

Reject duplicate or missing required sections, nonzero reserved fields or
flags, unsupported versions, invalid ranges, and unknown section kinds. A
runtime must not silently skip data it does not understand.

### Module metadata section (kind 3)

This section is exactly 8 bytes: the entry function ID (`u32`), target pointer
width (`u8`, either `32` or `64`), and three reserved zero bytes. The ID must
refer to a defined function with zero parameters and no result. If any constant,
local, parameter, or function result uses `isize` or `usize`, the target pointer
width must match the runtime. Otherwise either supported pointer width is
valid.

### Constants section (kind 1)

The section starts with a `u32` constant count, followed by exactly that many
entries. Each entry is a one-byte tag followed by its payload:

| Tag | Type | Payload |
|---:|---|---|
| `1` | `u1` | 1 byte: `0` or `1` |
| `2` | `i8` | 1-byte two's-complement integer |
| `3` | `i16` | 2-byte two's-complement integer, little-endian |
| `4` | `i32` | 4-byte two's-complement integer, little-endian |
| `5` | `i64` | 8-byte two's-complement integer, little-endian |
| `6` | `i128` | 16-byte two's-complement integer, little-endian |
| `7` | `isize` | 4 or 8 bytes, per target pointer width |
| `8` | `u8` | 1-byte unsigned integer |
| `9` | `u16` | 2-byte unsigned integer, little-endian |
| `10` | `u32` | 4-byte unsigned integer, little-endian |
| `11` | `u64` | 8-byte unsigned integer, little-endian |
| `12` | `u128` | 16-byte unsigned integer, little-endian |
| `13` | `usize` | 4 or 8 bytes, per target pointer width |
| `14` | `str` | `u32` byte length followed by UTF-8 bytes |

The `str` payload must contain valid UTF-8. Other `u1` encodings and unknown
tags are invalid. Constants are immutable and indexed from zero in their
encoded order. A compiler should deduplicate repeated constants where
practical; this is not required for validity.

### Functions section (kind 2)

The section starts with a `u32` function count, followed by that many function
records. Each record has this layout:

| Size | Field | Meaning |
|---:|---|---|
| 4 | Function ID | Unique `u32` identifier within this module |
| 2 | Parameter count | Number of leading local slots that are parameters |
| 2 | Local count | Total local slots, including parameters |
| 1 | Return type | Type code: `0` = no result; see type-code table below |
| 3 | Reserved | Must be zero |
| 4 | Code length | Number of bytes in this function's instruction stream |
| local count | Local types | One type byte per local slot; see type-code table below |
| code length | Instructions | Encoded instruction stream |

Type codes are `0` = no result (valid only for function results), `1` = `u1`,
`2` = `i8`, `3` = `i16`, `4` = `i32`, `5` = `i64`, `6` = `i128`,
`7` = `isize`, `8` = `u8`, `9` = `u16`, `10` = `u32`, `11` = `u64`,
`12` = `u128`, `13` = `usize`, and `14` = `str`. Local slots cannot have
type code `0`.

The function count must be at least one because the module must define an
entry function. Function IDs need not be contiguous. IDs must be unique,
parameter count must not exceed local count, every local and return type must
be recognized, and each record must fit entirely in the section. Code length
must be nonzero. A function's parameter types are the types of its first
`parameter count` local slots. Functions are identified by ID, not by
source-level name.

## Instruction set

Each instruction begins with a one-byte opcode. Operands immediately follow
the opcode in the stated encoding; instructions have no implicit alignment.
The stack is typed. `T` in the stack effect column means the specified type;
the rightmost item is the top of the stack.

| Opcode | Instruction | Operand | Stack effect |
|---:|---|---|---|
| `0x01` | `CONST` | constant index: `u32` | `[] -> [constant type]` |
| `0x02` | `LOAD_LOCAL` | local index: `u16` | `[] -> [local type]` |
| `0x03` | `STORE_LOCAL` | local index: `u16` | `[local type] -> []` |
| `0x04` | `POP` | none | `[T] -> []` |
| `0x10` | `ADD_INT` | none | `[T, T] -> [T]` |
| `0x11` | `SUB_INT` | none | `[T, T] -> [T]` |
| `0x12` | `MUL_INT` | none | `[T, T] -> [T]` |
| `0x13` | `DIV_INT` | none | `[T, T] -> [T]` |
| `0x14` | `REM_INT` | none | `[T, T] -> [T]` |
| `0x18` | `EQ_INT` | none | `[T, T] -> [u1]` |
| `0x19` | `NE_INT` | none | `[T, T] -> [u1]` |
| `0x1a` | `LT_INT` | none | `[T, T] -> [u1]` |
| `0x1b` | `LE_INT` | none | `[T, T] -> [u1]` |
| `0x1c` | `GT_INT` | none | `[T, T] -> [u1]` |
| `0x1d` | `GE_INT` | none | `[T, T] -> [u1]` |
| `0x20` | `JUMP` | signed relative offset: `i32` | unchanged |
| `0x21` | `JUMP_IF_ZERO` | signed relative offset: `i32` | `[u1] -> []` |
| `0x30` | `CALL` | function ID: `u32` | arguments -> optional result |
| `0x31` | `RETURN` | none | `[] -> []` |
| `0x32` | `RETURN_VALUE` | none | `[function return type] -> []` |

`T` in integer operations may be any integer type, including `u1` only for
comparisons; arithmetic accepts every integer type except `u1`. The types of the two operands must match exactly. For binary operations, the
first operand pushed is the left operand and the second is the right operand.
`CALL` consumes arguments in parameter order: the caller pushes the first
argument, then the second, and so on. The callee's return value, if any, is
pushed onto the caller's stack.

Each call frame has its own local slots and an operand-stack base. The callee
starts with an empty operand stack above that base; values already on the
caller's stack remain below it. At a call, arguments are removed from the
caller's stack and initialize the callee's parameter slots in declaration
order. On return, the callee must leave no extra operand values: a no-result
function leaves none, and a result-bearing function leaves exactly its single
result value. The runtime restores the caller's stack and pushes that result,
if present. Stack validation and return-height checks are relative to the
current frame's operand-stack base. The configured operand-stack limit counts
values across all active frames.

Jump offsets are signed two's-complement `i32` byte offsets relative to the
first byte after the jump instruction and its operand. A jump target must be
the first byte of an instruction in the same function. Falling off the end of
a function is invalid.

`RETURN` is valid only in a function with no result; `RETURN_VALUE` is valid
only in a function with a result and must return a value matching its declared
type. A function with a result must return a value on every reachable exit
path. A function without a result must return without a value.

### Numeric behavior

Integer arithmetic uses checked fixed-width behavior. Reject overflow,
division by zero, signed minimum divided by negative one, and signed minimum
remainder negative one as execution traps. Signed division truncates toward
zero; signed remainder has the sign of its left operand. `u1` values are
restricted to `0` and `1`; `JUMP_IF_ZERO` branches when its popped value is
`0`. Integer comparisons interpret signed types as signed and unsigned types
as unsigned. `str` values are immutable references to UTF-8 bytes in the
module's constants section; the runtime does not allocate or mutate guest
strings.

## Implementation readiness and examples

This specification is considered complete enough for implementation when the
runtime and compiler validate the following examples and invariants together:

- A minimal valid module: a no-result entry function with `RETURN` only and no
  constants.
- A valid module with a nonzero constant table and a function that loads a
  constant into a local and returns it.
- A malformed module with a duplicate section kind.
- A malformed module with a truncated constant payload.
- A malformed module whose jump target is not the start of an instruction.
- A validation error for a mismatched stack type at a control-flow join.
- A validation error for a function that returns a value in a no-result
  function, or omits a value in a result-bearing function.
- A resource-limit failure when the instruction count or call stack exceeds the
  configured limit.

These examples should live in the runtime test suite and become the acceptance
criteria before the ABI is treated as stable.

### Minimal valid module example

The following complete module is 85 bytes, with offsets measured from byte
zero. Each row is a contiguous byte sequence; the comments label its contents
and are not part of the module:

```text
55 52 4b 42  01 00  00 00  00 00 00 00  03 00  00 00  # header
01 00  00 00  34 00 00 00  04 00 00 00                 # constants dir
02 00  00 00  38 00 00 00  15 00 00 00                 # functions dir
03 00  00 00  4d 00 00 00  08 00 00 00                 # metadata dir
00 00 00 00                                             # constants: count 0
01 00 00 00                                             # functions: count 1
01 00 00 00  00 00  00 00  00  00 00 00  01 00 00 00  31
                                                        # function 1: no args,
                                                        # no locals, no result,
                                                        # code length 1, RETURN
01 00 00 00  40 00 00 00                                # entry 1, width 64
```

This is the smallest possible valid entry program. The function is valid
because it has zero parameters, no result, and a legal `RETURN` instruction for
that signature.

### Common malformed examples

The following inputs are invalid and must be rejected before execution:

- A constants section with a `str` payload whose UTF-8 bytes are malformed.
- A functions section with a duplicate function ID.
- A function record whose `parameter count` exceeds `local count`.
- A function whose return type is not recognized by the type-code table.
- A function whose `CALL` target is an unknown function ID.
- A `JUMP` or `JUMP_IF_ZERO` operand whose target is not the first byte of an
  instruction in the same function.
- A function whose exit paths reach different stack states at the same
  instruction.
- A module whose section directory contains overlapping or out-of-bounds
  ranges.

These examples describe the minimum set of parser and validator failures that
must be reproducible in tests before the runtime is considered ready.

## Validation before execution

The runtime must validate the complete module before running the entry
function:

1. Check the header, supported version, section directory, ranges, uniqueness,
   complete section coverage, and exact section consumption.
2. Parse constants, module metadata, and function records with checked reads
   and checked size arithmetic.
3. Decode every function's entire instruction stream. Reject unknown opcodes,
   truncated operands, invalid constant/local/function references, and jump
   targets that are not instruction boundaries.
4. Validate instruction stack types and control-flow joins. Every control-flow
   path reaching the same instruction must have the same stack types and
   height. Reject underflow, incompatible types, invalid returns, and
   inconsistent stack states. Decode all instructions, including unreachable
   ones; perform stack analysis for reachable instructions.
5. Validate calls against the target function's parameter and return types,
   including argument order and result stack effect. Recursion is valid and is
   bounded by the call-stack limit.

No guest instruction may execute until validation succeeds. At execution,
local slots begin uninitialized except for parameter slots. `LOAD_LOCAL` of an
uninitialized slot is a runtime trap as a defense against invalid bytecode;
the source compiler must reject any source path that could read a container
before assigning it. `STORE_LOCAL` initializes or replaces a slot with a value
of its declared type.

## Execution limits and errors

The implementation must enforce configurable limits for module size,
instruction count (fuel), value-stack depth, local slots per function, and
call-stack depth. The command-line options above configure these limits; omitted options use
their defaults. If an embedding API is provided, it must expose the same
limits. Limits must be checked before allocation or before exceeding the
configured amount. Local slot limits include parameter slots; call depth
counts the entry function as one active call. Do not let an untrusted module
disable limits or raise them.

At a function return, the operand stack must contain exactly the return value
for a function with a result, or be empty for a function without a result.
The runtime counts every executed instruction, including instructions in
called functions, against the run's fuel budget. Function and instruction
locations in diagnostics use the absolute byte offset of the instruction in
the module file; if an error has no associated instruction, report the
function ID or offset only when known.

Errors must be reported explicitly and distinguish at least malformed module,
unsupported version or opcode, validation failure, resource limit exceeded,
uninitialized local, and execution trap. Include a function ID and byte offset
when the location is known. Do not turn a runtime error into a successful
result, and do not use panics as guest error handling.

## Rust implementation guidance

- Keep decoding and validation separate from execution. Use a checked reader
  over the module byte slice; do not transmute bytes into Rust structs.
- Represent validated instructions and function metadata with Rust types so
  the execution loop does not repeatedly trust raw offsets or indices.
- Use explicit error types and propagate errors to the executable boundary.
  Avoid `unwrap` and `expect` on module-controlled data.
- Keep the runtime deterministic for a given module and configured limits.
  Do not add filesystem, network, process, or other host access to bytecode
  without a separately reviewed capability design.
- Add tests for valid modules, malformed/truncated sections, invalid indices,
  bad jumps, stack/type mismatches, invalid returns, and resource-limit
  enforcement before treating the runtime as ready.

## Status and compatibility

This is a documented target, not an implemented executable or frozen public
ABI. Version 1.0 is the initial implementation target. Until the runtime and
compiler implement and test this contract together, treat the exact encoding
and instruction semantics as subject to revision.

Within a released major version, existing opcode meanings and encodings must
not change. A runtime must reject a module with an unsupported major or minor
version rather than guessing how to interpret it. Any incompatible format or
semantic change requires a new major version.

### Author decisions before ABI freeze

These choices are specified as an initial implementation target above, but
should be confirmed before declaring the bytecode ABI frozen:

- Should the runtime's command-line option names and defaults be the public
  interface, or should the runtime manager/service center configure limits
  through another interface?
- Is rejecting section gaps and padding required, or should a future-compatible
  format permit ignored padding between the three known sections?
- Must unreachable instructions receive full semantic validation (for example,
  rejecting a `RETURN` with the wrong function result type), or is decoding
  unreachable instructions and validating only reachable stack/control-flow
  behavior sufficient?
- Are process exit-code values intentionally unspecified, or should the
  executable contract assign stable codes before implementation?
- When the fuel limit is reached, should the next instruction be rejected
  before execution, and should that attempted instruction count toward the
  reported fuel usage?
