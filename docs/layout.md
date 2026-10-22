# Urk

The language of the future.

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
consumes bytecode. Do not assume details of the bytecode format or the
communication between the three programs unless they are specified elsewhere.
