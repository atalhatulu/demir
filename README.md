# Demir

> A modern systems programming language focused on safety, clarity, predictable semantics, and native performance.

**Demir** is an experimental general-purpose programming language and compiler project. It explores a modern programming model built around compile-time safety, readable code, deterministic semantics, and native execution.

Source files use the `.dmr` extension.

---

## Why Demir?

Existing languages already solve many individual problems extremely well. Demir explores a different combination:

- Compile-time memory safety
- Ownership and borrowing
- Explicit mutability
- Predictable semantics
- Native performance
- Strong compiler diagnostics
- Compiler-level program analysis

The goal is not to make another Rust, C++, or Go.

> **Make programs easier to understand, harder to break, and fast enough to run close to the machine.**

---

## Example

```demir
fn main() {
    let a = 20;
    let b = 22;

    print(a + b);
}
```

Output:

```text
42
```

Demir syntax is designed for **semantic clarity and low cognitive overhead**, rather than trying to imitate natural language.

---

## Compiler Architecture

Demir currently uses **Rust as its bootstrap implementation language**. Rust is not Demir itself.

The compiler pipeline is designed around:

```text
.dmr Source
    ↓
Lexer
    ↓
Parser
    ↓
AST
    ↓
Semantic Analysis
    ↓
Typed IR / SSA
    ↓
Optimization
    ↓
Monomorphization
    ↓
Cranelift
    ↓
Native Executable
```

Cranelift handles native code generation while Demir focuses on language semantics, analysis, and its intermediate representation.

Detailed architecture is documented in [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## Current Status

**V0.1 — Compiler Development**

Established foundations:

- [x] Rust bootstrap compiler
- [x] Parser / AST foundation
- [x] Variables and assignment
- [x] Arithmetic expressions
- [x] Blocks and conditionals
- [x] Borrow checker foundation
- [x] Cranelift integration
- [x] Native object generation
- [x] Native executable generation

In development:

- [ ] Advanced type system
- [ ] Ownership / borrowing completion
- [ ] Lifetimes
- [ ] Generics and monomorphization
- [ ] Traits / interfaces
- [ ] Module system
- [ ] CFG / SSA pipeline
- [ ] Foundational optimizations
- [ ] IR interpreter
- [ ] Runtime and standard library
- [ ] Debug information

---

## Core Ideas

### Safety by default

Demir is designed around compile-time guarantees for ownership, borrowing, lifetimes, type safety, and controlled mutability.

### Explicit mutability

```demir
let value = 10;

var counter = 0;
counter = 1;
```

### Explicit failure handling

Optional values use:

```text
Option<T>
```

Operations that can fail use:

```text
Result<T, E>
```

### Native execution

The goal is a native executable without requiring a heavyweight managed runtime for ordinary programs.

---

## Roadmap

### V0.1 — First Real Compiler

Complete the path from `.dmr` source to a correct native executable on **x86-64 Linux**.

### V0.2 — Language Expansion

- Richer type system
- Structs
- Traits / interfaces
- Modules
- Generics
- Standard library
- Better diagnostics and optimization

### V0.3+

- Advanced compiler tooling
- Ownership / data-flow visualization
- Concurrency models
- Additional targets
- AI-oriented tooling

### V2+

**Self-hosting Demir compiler.**

```text
Rust bootstrap compiler
        ↓
Demir compiler
        ↓
Demir compiler written in Demir
```

---

## AI Direction

AI is a **future capability**, not a requirement for executing Demir programs.

Potential areas include:

- Agent-oriented programming
- Intent-aware compilation
- LLM bindings
- AI-assisted diagnostics
- Machine-readable program semantics

The core language remains deterministic and usable without an AI system.

---

## Documentation

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — Compiler architecture and technical decisions
- [`help.md`](help.md) — Implementation specification and development guide

---

## Design Principles

1. **Correctness before cleverness.**
2. **Safety should be enforced at compile time whenever possible.**
3. **Semantics must be deterministic.**
4. **Syntax should optimize for reading and maintenance.**
5. **Native performance is a core requirement.**
6. **Compiler stages should have clear responsibilities.**
7. **Intermediate representations must be analyzable and testable.**
8. **Unsafe behavior must be explicit.**
9. **AI should enhance the programming model, not replace deterministic semantics.**
10. **Build the compiler before building the ecosystem.**

---

## Vision

Demir starts with a compiler.

The long-term vision is a programming environment where software is:

**safe enough to trust, simple enough to understand, predictable enough to debug, and fast enough to run natively.**

The first milestone is simple:

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

---

## License

License information will be added as the project is finalized.
