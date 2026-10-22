# Language specification

This file is the language-design reference for the AI assistant implementing
the language. The project author will define the language rules here; use this
document as the source of truth when designing or changing the lexer, parser,
runtime, tests, and related documentation.

Do not invent or assume language behavior that is not specified here. If an
implementation decision depends on an unspecified rule, ask the project
author before deciding.

## core language
Urk uses a **center** syntax to execute code, the value at the center of the line is modded via the contiton before it and the moding coming after, 
```text
    <contiton><value><opperator,function,ect>
```
this is just how you edit code however, defining code requires using template objects, these templates are values copiable objects that are used to give data a owner untill it is passed into a real variable of sorts, or a function, example, the `container` object is a template, this template gives data the attributes of a variable, however it holds no data on its own other than a placeholder name, 
```text
container makes "<name>"
```
the reason we need a template is because data cannot be fed into a variable or anything unless it is having an owner, if it has no owner at time of being made it will be forgotten. using a container to pass a value into a function also works

```text
container makes "y"
when 12 uses "y" for my_function
```
this is an odd example, because the syntax of the language is being shown, but right now you can read it as, 
```text
<making y>
when 
#the contiton that checks nothing is going wrong
12 
# a value
uses 
# assigns "y" to 12, thus giving it a owner
for 
# a word that passes a value into a function
my_function 
#the function
```

now you have a good idea of how data works, lets use a bit more of a complitcated example. In this example, we will make a function, called guess, guess will take 3 values
- a value that will be multiplyed
- a value that the first is being multiplyed by
- the expected output
this function returns a boolean of weather the guessed output is corrected.

```text
when functiontemp makes "guessing_game" 
"guessing_game" uses (num1 as f64, num2 as f64, guess as f64 )
guessing_game returns bool for {
    if num1 * num2 != guess false uses bool for return
    if num1*num2 ==guess true uses bool for return

```