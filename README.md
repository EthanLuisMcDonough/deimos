# Deimos Compiler

Deimos is a C-style language that takes inspiration from Rust and Pascal. The Deimos compiler is a non-optimizing compiler that targets MIPS assembly, meaning it is expected to take source code files and turn them into assembly files that can be run in MARS. This project was developed over the course of a few months as a final project for CMPEN351.

## Implemented features

- Functions  
- If/elif/else statements
- While loops
- Signed and unsigned 32-bit integers
- 8 bit `byte` type
- 32-bit floating point numbers
- Arrays
- Type casts
- Arithmetic operations (`+`, `-`, `*`, `/`, `%`) for integer and byte types. All except for `%` can be applied to floating point types as well
- Bitwise/logical AND and OR
- Logical operators (`==`, `!=`, `<`, `>`, `>=`, `<=`) for all numeric values
- Pointers and pointer arithmetic
- Global variables
- Low-level interfaces (ASM block and `syscall` construct)
- 16 standard library functions that give access to MARS functionality (e.g. `mars_sleep` and `mars_midi_sync`). These functions are automatically included in Deimos programs

## Examples

The following code is an implementation of the Towers of Hanoi algorithm in Deimos:

```
sub towers_of_hanoi(n: i32, from: i32, to: i32, other: i32) {
   if (n > 0) {
  	call towers_of_hanoi(n - 1, from, other, to);
  	print "Move D", n, " from R", from, " to R", to, "\n";
  	call towers_of_hanoi(n - 1, other, to, from);
   }
}

program {
   call towers_of_hanoi(3, 1, 2, 3);
}
```

Example programs can be found in the [./samples](./samples) directory. These examples include the FizzBuzz problem (loops and nested control flow), a program that plays random MIDI noises (plays sounds and pauses), an ASCII mandelbrot (floating point operations), and the Towers of Hanoi implementation (recursion) shown in this document.

## Architecture description

The deimos compiler is a Rust project that consists of five crates (packages): `deimos`, `deimos_ast`, `deimos_parser`, `deimos_codegen`, and `mips_builder`. The `deimos` package is the executable driver that invokes the other crates and `deimos_ast` contains the structures that represent different language constructs in the syntax tree. The `deimos_parser` crate is responsible for lexing and parsing source files, and `mips_builder` is used for generating MIPS source code. The `deimos_codegen` package traverses the AST and generates the output MIPS assembly. Any semantic errors are detected during code generation.

The lexer works by collecting characters into lexemes until it finds a character that "breaks" the lexeme’s pattern. Execution will terminate with an error if the lexer encounters an invalid character. The parser is LL(1) and works by expecting certain patterns based on signifying keywords (e.g. `if`, `while`, `sub`). Expressions are parsed separately using the shunting-yard algorithm. If the compiler encounters an invalid sequence of characters, an error will be thrown and the compilation will stop. The codegen and semantic analysis are folded into one step so that types and identifiers can be checked without requiring another pass to look those same values up again. This compiler doesn’t have any optimizing passes or flattened intermediate representation.

Most of the codegen effort was concentrated into writing the codegen for statements and expressions. Statements that contain nested code (if and while blocks) can be processed recursively. Each if statement and while loop will have a special counter id to differentiate it in MIPS from other statement labels.

Expressions differ from C in that they cannot have function calls in them. They are also (mostly) branchless. None of the logical boolean operators branch or "short circuit". Floating point operators do call a branching mini-function that gets the floating point condition by using bc1f/bc1t.

Registers are delegated on a first come first serve basis. A register can’t be used in code generation until it is freed. Registers are “freed” whenever two values are combined into one or whenever a value in a register of one type (regular vs floating point) is moved to a register of a different type (float to int casting and vice-versa). If the code generator finds that all registers are occupied, it can create a virtual register. Virtual registers are word sized values on the stack that can store expression values. They are always negative multiples of four, so virtual register 0 corresponds with `-4($sp)`, virtual register 1 is `-8($sp)`, and so on. All other stack references are done with positive offsets, so these virtual registers should never interfere with stack variables as long as the stack pointer is kept up to date. Virtual register values are loaded into designated “temp registers” (`$t8`, `$t9`, `$f18`, and `$f19`). They are read and/or written to and discarded promptly.

Whenever a function is called, N word stack slots are added to the stack, where N is the number of function arguments. Each argument is evaluated and written to its respective slot before `jal` is called. The function body can reference these stack values as well as the local variables defined in the function body. Once the function body has finished running, the stack pointer is reset and the saved return address is saved and jumped to.

## Build instructions

Building Deimos requires the Rust programming language and the Cargo build tool. Once these are installed, Deimos can be built by entering the driver package folder (`./deimos`) and running cargo build:

```shell
git clone https://github.com/EthanLuisMcDonough/deimos
cd deimos/deimos
cargo build
```

The source code file is passed in via the command line interface and the output file can be named using the `-o` flag. The compiler lex and parse stages can be tested by using the `-debug-stage=lex` and `-debug-stage=parse` flags respectively.
