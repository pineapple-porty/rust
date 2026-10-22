# Urk
The language of the future
## file data
  files in Urk all end with `.urk`.
  A file in urk has two parts.
```text
  <!#command hint>
  <code>
```
Urk lets you (and makes you) write the code that is used to parse the file. Using the command hint, the command hint is a way of feeding values into the `runtimemanager.exe` that works to give the `runtime.exe` code to run, the `runtime.exe` only reads bytecode, so the runtimemanager takes that normal code you've written, and turns into runtime readable code.
## the command hint
see commandhint.md for more info
# the code
biggest part
## keywords
### objects (pre defined)
`container` : variable template object, holds a value until a variable claims it, then it is removed from the stack, immutable
`\` :