# Urk

The language of the future.

## Project implementation

The project is intended to be implemented in Rust. The service center,
runtime manager, and runtime are planned Rust components that work together
to run Urk programs. Rust is the implementation language for the tools and
runtime, not the language users write their programs in: Urk source files use
Urk syntax, and the runtime is intended to execute bytecode produced from that
source.

The repository currently contains the Rust package in
[`lang_prj/`](../lang_prj/), with its Cargo manifest and source under that
directory. The three named programs below describe the planned execution
flow; they are not yet three implemented executables. As the project grows,
keep Rust source and Cargo manifests in the relevant crate directories.

## How Urk code runs

Urk uses three programs to run a source program:

- `urkservicecenter.exe` is the user-facing entry point. Start it with the
  path to the Urk program you want to run.
- `runtimemanager.exe` prepares the program for execution by translating its
  source code into bytecode.
- `runtime.exe` executes bytecode. It does not execute Urk source code
  directly.

The intended execution flow is:

1. The user starts `urkservicecenter.exe` and provides a program path.
2. The service center sends the program through `runtimemanager.exe`.
3. The runtime manager converts the source code into bytecode and supplies it
   to `runtime.exe`.
4. The runtime executes the bytecode.

For example, from PowerShell:

```powershell
urkservicecenter.exe "C:\path\to\program.urk"
```

The service center is the normal way to run a Urk source program. Running
`runtime.exe` directly is only appropriate when supplying bytecode; passing
Urk source code directly to it will not work.

## Source code and bytecode

Bytecode is the representation that `runtime.exe` executes. Urk source code
describes the bytecode the runtime should run, and `runtimemanager.exe`
translates the source into that representation.

When implementing or changing the language, preserve this distinction:
source-code processing belongs before runtime execution, and the runtime
consumes bytecode. The version 1 bytecode module format and runtime behavior
are documented in [runtime.md](./runtime.md). The source-language rules and
implementation requirements are documented in [syntax.md](./syntax.md). The
service center's communication with the runtime manager remains unspecified
and must be designed before those programs can be integrated end to end.
