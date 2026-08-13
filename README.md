# Demir

> A modern systems programming language focused on safety, clarity, predictable semantics, and native performance.

> **⏸️ ARCHIVED — project on hold (Aug 2026).** Demir reached a self-contained working milestone and is now shelved. The repo (full commit history, compiler source, tests, examples) remains on GitHub and is fully re-cloneable; see [Project Status 2026](#project-status-2026-archival) below for what was accomplished, why it was set aside, and how to pick it back up.

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

**V0.1 — Compiler Development (archived milestone)**

Working end-to-end today (`.dmr` → native executable, x86-64 Linux, via JIT or AOT/Cranelift):

- [x] Rust bootstrap compiler
- [x] Parser / AST foundation
- [x] Variables and assignment
- [x] Arithmetic expressions
- [x] Blocks and conditionals (`if`, `while`)
- [x] Structs (declaration, literal instantiation, field access)
- [x] Functions with parameters, returns, and struct-return value-copy into the caller frame
- [x] Borrow checker foundation (ownership/move tracking)
- [x] Pointer / reference codegen (`&`, `&mut`, `*`) — stack-slot based, incl. `&mut` params + deref-store
- [x] Design by contract (`requires` / `ensures`) — runtime enforcement, JIT and AOT
- [x] CFG + SSA pipeline (dominance, phi placement, renaming)
- [x] Cranelift integration, native object + executable generation

Safety hardening shipped in the final commits (guards added so silent data-corruption is rejected loudly instead of producing garbage):

- `&struct` (borrowing a whole struct) → **compile error** (the borrow/pointer model only supports borrowing primitives for now)
- Parser fix: `Identifier {` is only treated as a struct literal when the identifier is a known struct name — `while i < n {` and `if flag {` (identifier-terminated control conditions) now parse correctly
- `MirFunction.ssa_origin` maps each SSA-renamed local back to its pre-SSA id, so any future codegen pass can resolve a renamed local's type — the SSA/type-loss wall future features would hit is now gone

Not yet implemented (out of scope for this milestone):

- [ ] Generics / monomorphization
- [ ] Lifetimes
- [ ] Traits / interfaces
- [ ] Module system
- [ ] Arrays and a standard library (the IR runtime path is graph-ready but arrays were not added)
- [ ] Compile-time contract proving (runtime-only today)
- [ ] Foundational optimizations beyond what Cranelift provides
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

## Project Status 2026 (Archival)

**Demir is shelved as of August 2026.** This is not a failed project — it is a deliberately concluded milestone, parked on GitHub.

### What was accomplished
A working compiler in Rust that turns `.dmr` source into correct native x86-64 Linux executables through a real pipeline: `lexer → parser → AST → semantic analysis/borrowck → MIR → CFG + SSA → Cranelift → object/executable`. It compiles its own test suite and a `fib(10)` benchmark (JIT and AOT both print `55`). Struct return works end-to-end (value-copied into the caller frame), borrowing/pointers work for primitives, and design-by-contract (`requires`/`ensures`) is enforced at runtime. The final commits were hardening passes that replaced several silent data-corruption paths with loud compile-time errors.

### Why it was set aside
The language reached a coherent, self-contained milestone but faced the classic two walls of any solo language project: **no standard library / collections and no ecosystem**, and therefore no practical users. Every hour spent adding features (arrays, generics, modules) compounds costs without yielding a reachable near-term outcome. The motivating question — *"what genuinely new thing does this give people?"* — narrowed to a single defensible thesis: **a contract-first language whose guarantees make AI-generated code safer to trust**. That thesis is real but bigger than a solo timeframe, so the project was parked rather than half-developed.

### How to pick it back up
- Clone: `git clone git@github.com:atalhatulu/demir.git`
- Build: `env -u PYTHONPATH cargo build --release` (Rust 2021, Cranelift)
- Run: `./target/release/demir examples/fib.dmr` (JIT) — the pipeline also AOT-compiles to `output.o` and links a standalone `./output` executable
- The next milestone on this thesis, if resumed, is **arrays** (the last gap before "loop + collection" programs are possible), wired to `requires index < len` as an implicit bounds-check contract.

This README, `docs/`, `ARCHITECTURE.md`, and `help.md` document the design and the reasoning behind each stage, so the project can be resumed without re-deriving past decisions.

---

## License

License information will be added as the project is finalized.
