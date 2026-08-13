# Demir

> A modern systems programming language focused on safety, clarity, predictable semantics, and native performance.

**Demir** is an experimental general-purpose programming language and compiler project. It explores a modern programming model built around compile-time safety, readable code, deterministic semantics, and native execution.

Source files use the `.dmr` extension.

---

## Why Demir?

Existing languages already solve many individual problems extremely well. Demir explores a different combination:

- Compile-time memory safety
- Ownership and borrowing
- **Design by contract** (`requires` / `ensures`), enforced at runtime today, compile-time proving planned
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

### Design by contract (working today)

Functions can declare **pre-conditions** (`requires`) and **post-conditions** (`ensures`). Pre-conditions are emitted as runtime checks and fail fast with a clear message and a non-zero exit status if violated.

```demir
fn divide(a: Int, b: Int) -> Int
    requires b != 0
{
    return a / b;
}
```

Calling `divide(10, 0)` verifies the `requires b != 0` pre-condition at runtime and aborts:

```text
ASSERT FAILED: pre-condition 1 failed
```

This is checked in both JIT and AOT paths. Compile-time verification (statically proving a contract holds) is a future research direction.

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
- [x] Borrow checker foundation (ownership/move tracking + stack-slot based `&`/`&mut`/`*` pointer codegen)
- [x] Cranelift integration
- [x] Native object generation
- [x] Native executable generation

In development:

- [x] Design by contract (`requires` / `ensures`) — runtime enforcement
- [x] Pointer / reference codegen (`&`, `&mut`, `*`) — stack-slot based
- [ ] Advanced type system
- [ ] Ownership / borrowing completion
- [ ] Lifetimes
- [ ] Memory model (arena / region-based — proposed)
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

- **Memory model: arena / region-based** (proposed) — region-based allocation with scoped lifetimes, chosen over a general heap GC to keep semantics deterministic and predictable
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

## Research Direction (AI, future / V0.3+)

AI integration is **future research**, not a requirement for executing Demir programs. Today's compiler has only lexical groundwork (keywords such as `intent`, `agent`, `capability`, `ask`); there is **no AI runtime, no natural-language parsing, and no AI-first architecture** in the current compiler. This section describes where we would like to go, not what exists today.

Potential future research areas:

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
