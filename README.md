# Demir

> A modern systems programming language focused on safety, clarity, predictable semantics, and native performance.

**Demir** is an experimental general-purpose programming language and compiler project. Its goal is to explore what a modern systems language can look like when compile-time safety, readable source code, deterministic behavior, and native execution are treated as first-class design constraints.

Demir source files use the `.dmr` extension.

---

## Why Demir?

Modern languages already solve many individual problems extremely well. Demir is not being built simply to create another syntax for the same ideas.

The project explores a compiler architecture where:

- memory safety is enforced at compile time
- ownership and borrowing are part of the language model
- control flow is represented explicitly through CFG and SSA
- generic code is resolved at compile time through monomorphization
- optimizations operate on a structured intermediate representation
- the same compiler pipeline can support interpretation and native execution
- native binaries are produced without requiring a large managed runtime
- diagnostics are treated as a core part of the language experience
- future tooling can reason about program semantics rather than only source text

The long-term objective is simple:

> **Make programs easier to understand, harder to break, and fast enough to run close to the machine.**

Demir is not intended to replace every language immediately. It is a research and engineering project aimed at building a coherent language and compiler architecture from the ground up.

---

## Current Status

**Version:** V0.1 development

Demir is currently in the compiler implementation phase.

Implemented or actively established in the current architecture include:

- [x] Rust bootstrap compiler
- [x] Lexer / parser foundation
- [x] AST → Cranelift integration
- [x] JIT execution foundation
- [x] Variables and assignment
- [x] Arithmetic expressions
- [x] Blocks and conditionals
- [x] Borrow checker architecture
- [x] Cranelift object generation
- [x] Native executable generation
- [ ] Advanced type system
- [ ] Structs
- [ ] Traits / interfaces
- [ ] Generics and monomorphization
- [ ] Module system
- [ ] Complete ownership and lifetime checking
- [ ] CFG / dominator / dominance-frontier pipeline
- [ ] SSA construction and optimization
- [ ] IR interpreter
- [ ] DWARF debug information
- [ ] Minimal standard library
- [ ] AI-oriented language features
- [ ] LLM-native bindings

The checklist represents the project direction; individual components may still be under active development.

---

## Hello, Demir

A minimal Demir program is designed to be intentionally familiar:

```demir
fn main() {
    let a = 20;
    let b = 22;

    print(a + b);
}
```

Expected output:

```text
42
```

The syntax is deliberately not designed to imitate natural language. Demir optimizes for **semantic clarity and low cognitive overhead** when code is read, debugged, maintained, and modified.

---

## Language Philosophy

### 1. Safety by default

Unsafe behavior should never be hidden behind convenient syntax.

The language is designed around compile-time guarantees including:

- ownership
- borrowing
- lifetime correctness
- type safety
- controlled mutability
- null safety through `Option<T>`
- explicit error handling through `Result<T, E>`

### 2. Explicit mutability

Immutable bindings are the default:

```demir
let value = 10;
```

Mutable state is explicit:

```demir
var value = 10;
value = 20;
```

### 3. No implicit dangerous conversions

For example:

```demir
let value: Int = "hello";
```

must fail during compilation rather than becoming a runtime surprise.

### 4. Explicit failure handling

Instead of relying on a traditional nullable value model:

```demir
Option<T>
```

represents optional values, while:

```demir
Result<T, E>
```

represents operations that may succeed or fail.

### 5. Native execution

Demir is designed to produce native executables rather than requiring a heavyweight managed runtime for ordinary programs.

---

## Compiler Architecture

The current compiler architecture is built around a staged compilation pipeline:

```text
                 Demir Source (.dmr)
                         │
                         ▼
                      Lexer
                         │
                         ▼
                      Parser
                         │
                         ▼
                        AST
                         │
                         ▼
               Semantic Analysis
                         │
                         ▼
                    Typed AST
                         │
                         ▼
                        CFG
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
        Dominator Tree       Dominance Frontier
              │                     │
              └──────────┬──────────┘
                         ▼
                        SSA
                         │
                         ▼
                    Optimization
                         │
                         ▼
                 Monomorphization
                         │
                         ▼
                  Concrete SSA / IR
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
        IR Interpreter          Cranelift
                                      │
                                      ▼
                                Object File
                                      │
                                      ▼
                                System Linker
                                      │
                                      ▼
                              Native Executable
```

The compiler itself is currently implemented in **Rust**. Rust is a bootstrap implementation language; it is not the Demir language itself.

Long-term self-hosting is a future goal rather than a V0.1 requirement.

---

## Why Cranelift?

Demir uses **Cranelift** as its native code generation backend.

This keeps the compiler focused on language semantics and its own intermediate representation while delegating machine-code generation to a mature backend.

The intended responsibility boundary is:

```text
Demir Compiler
      │
      ▼
Demir IR / SSA
      │
      ▼
Cranelift IR
      │
      ▼
Native Machine Code
```

This also allows the compiler to support fast execution paths without committing V0.1 to maintaining a custom x86-64 backend.

---

## Memory Model

Demir follows an ownership-oriented memory model.

The intended model is:

```text
Ownership
    +
Borrowing
    +
Stack
    +
Heap
    +
Deterministic lifetime
```

The compiler should reject problems such as:

- use-after-move
- use-after-free
- dangling references
- invalid aliasing
- conflicting mutable access

A tracing garbage collector is not required by the core language model.

An explicit `unsafe` escape hatch is planned for operations that cannot be proven safe by the normal compiler rules.

---

## Ownership Example

Conceptually:

```demir
let data = create();

consume(data);

print(data);
```

The final use of `data` should fail if `consume` takes ownership of it.

The compiler should produce a diagnostic similar to:

```text
value `data` was moved
```

The important principle is that the error is discovered **before the program runs**.

---

## Type System

The initial type system is designed around a small, predictable core.

Primitive types include:

```text
Int
Float
Bool
Char
String
Unit
```

Core generic types include:

```text
Array<T>
Option<T>
Result<T, E>
```

Future versions expand this foundation with structs, traits/interfaces, generic constraints, and additional abstractions.

---

## Generics and Monomorphization

Demir is designed to support generic programming without requiring dynamic dispatch for ordinary generic code.

Example:

```demir
fn identity<T>(value: T) -> T {
    return value;
}
```

Conceptually, compilation can specialize it into concrete instances such as:

```text
identity<Int>
identity<String>
```

The intended pipeline is:

```text
Generic Program
      │
      ▼
Generic SSA
      │
      ▼
Monomorphization
      │
      ▼
Concrete SSA
      │
      ▼
Cranelift
```

Dynamic trait objects and vtable-based dispatch are intentionally outside the V0.1 core scope.

---

## CFG and SSA

Control flow is not treated as an afterthought.

For example:

```demir
if condition {
    x = 10;
} else {
    x = 20;
}

print(x);
```

is represented internally through a control-flow graph and transformed into SSA form.

Conceptually:

```text
x0 = 10
x1 = 20
x2 = phi(x0, x1)
```

Phi nodes are compiler-internal constructs; they are never written by the programmer.

Loops must also support loop-carried SSA values and correctly placed Phi nodes.

This representation provides a foundation for reliable analysis and optimizations such as:

- constant folding
- constant propagation
- sparse conditional constant propagation
- dead-code elimination
- unreachable-block elimination
- CFG simplification
- SSA cleanup
- basic common-subexpression elimination

---

## Error Handling

The language favors explicit error values rather than hidden control flow.

Example:

```demir
fn read_file(path: String) -> Result<String, IOError>
```

Possible outcomes:

```text
Ok(data)
Err(error)
```

Pattern matching is intended to be exhaustively checked by the compiler.

---

## Runtime and ABI

V0.1 targets **x86-64 Linux**.

The runtime is intentionally minimal.

The planned system boundary is:

```text
Demir Program
     │
     ▼
Demir Runtime
     │
     ▼
C ABI
     │
     ▼
libc
     │
     ▼
Linux
```

The project does not currently aim to implement a custom operating-system interface or custom linker.

---

## Debugging

The first debugging strategy is based on standard native debugging infrastructure rather than a custom debugger.

The target architecture is:

```text
Demir Compiler
      │
      ▼
DWARF Debug Information
      │
      ├── GDB
      └── LLDB
```

The long-term vision may include compiler-aware tooling for ownership, value provenance, and SSA/data-flow visualization, but these are deliberately separated from the V0.1 compiler milestone.

---

## AI-First Direction

AI is a future design dimension of Demir, not a replacement for normal programming-language semantics.

Potential future areas include:

- agent-oriented language constructs
- intent-aware compilation
- structured LLM bindings
- machine-readable program semantics
- compiler diagnostics designed for both humans and coding agents
- native AI/LLM integration

The core principle is:

> AI features must build on a deterministic and well-defined language rather than making the language itself dependent on an LLM.

This distinction is important: a Demir program must remain a real program even when no AI system is involved.

---

## Development Roadmap

### V0.1 — First Real Compiler

The primary milestone is:

```text
.dmr source
   ↓
Demir compiler
   ↓
native executable
   ↓
Linux x86-64
```

Focus areas:

- lexer
- parser
- AST
- semantic analysis
- type checking
- ownership
- borrowing
- lifetimes
- CFG
- SSA
- foundational optimization
- monomorphization
- IR interpreter
- Cranelift backend
- runtime
- diagnostics
- tests

### V0.2 — Language Expansion

Potential areas:

- richer type system
- structs
- traits/interfaces
- modules
- improved generics
- stronger diagnostics
- standard library expansion
- improved optimization
- better tooling

### V0.3+ — Advanced Tooling and Runtime

Potential areas:

- advanced compiler-aware debugging
- ownership visualization
- SSA/data-flow visualization
- dynamic dispatch
- advanced async/concurrency models
- additional targets
- richer package ecosystem
- AI-native tooling

### V2+ — Self Hosting

A long-term goal is to make the compiler capable of being implemented in Demir itself:

```text
Rust bootstrap compiler
        ↓
Demir compiler
        ↓
Demir compiler written in Demir
        ↓
Self-hosted Demir toolchain
```

Self-hosting is intentionally not required for the first usable compiler.

---

## Project Structure

The repository is organized around the compiler rather than a large framework or runtime ecosystem.

```text
demir/
├── src/            # Compiler implementation
├── tests/          # Compiler and language tests
├── examples/       # Example Demir programs
├── runtime.c       # Minimal native runtime components
├── ARCHITECTURE.md # Detailed architecture and decisions
├── help.md         # Implementation specification / development guide
├── Cargo.toml      # Rust compiler project configuration
└── README.md       # Project overview
```

The exact structure may evolve as the compiler grows.

---

## Design Principles

Demir development follows a small set of strict principles:

1. **Correctness before cleverness.**
2. **Safety should be enforced at compile time whenever possible.**
3. **The language must have deterministic semantics.**
4. **Syntax should optimize for reading and maintenance, not natural-language imitation.**
5. **Native performance is a core requirement.**
6. **Compiler stages should have explicit responsibilities.**
7. **Intermediate representations should be analyzable and testable.**
8. **Unsafe behavior must be explicit.**
9. **AI must enhance the programming model, not replace deterministic semantics.**
10. **Do not build a giant ecosystem before the compiler itself is trustworthy.**

---

## Non-Goals for V0.1

To keep the first compiler focused, the following are intentionally out of scope unless they become necessary blockers:

- custom x86-64 backend
- custom register allocator
- custom instruction selector
- custom linker
- custom debugger
- dynamic trait objects
- vtables
- advanced async runtime
- actor runtime
- full package ecosystem
- IDE / GUI
- self-hosting compiler
- AArch64 backend
- WebAssembly backend
- Windows backend
- macOS backend

These may become future projects or roadmap items.

---

## Contributing

Demir is an experimental compiler project and its architecture is expected to evolve.

When contributing, prefer changes that:

- preserve clear compiler boundaries
- include tests for language semantics
- avoid unnecessary complexity
- document architectural decisions
- keep V0.1 focused on producing a correct native executable

Before introducing a major feature, consider whether it belongs in the current milestone or should remain a future roadmap item.

---

## License

License information will be added as the project is finalized.

---

## Vision

Demir starts with a compiler.

The larger goal is to explore a programming environment where a developer can write software that is:

**safe enough to trust, simple enough to understand, predictable enough to debug, and fast enough to run natively.**

The first objective is not to build everything.

It is to build one complete path:

```text
Demir source
     ↓
Correct semantics
     ↓
Safe IR
     ↓
Native code
     ↓
A real executable
```

Everything else comes after that.
