Kesinlikle. Artık Antigravity'ye **fikir tartışması değil, uygulanabilir bir teknik şartname** vermeliyiz. Aşağıdaki metni doğrudan Antigravity'ye gönderebilirsin.

````md
# LANGUAGE PROJECT — V0.1 IMPLEMENTATION SPECIFICATION

## STATUS

This document supersedes previous architectural drafts where they conflict.

The project is now entering the **implementation phase**.

The goal is NOT to design endlessly.

The goal is:

> Build the first real compiler and produce the first native executable.

Do not expand the scope unless a technical blocker requires it.

---

# 1. CORE PHILOSOPHY

The language does NOT need to imitate natural human language.

Do NOT optimize syntax for:

- English-like sentences
- conversational syntax
- natural-language resemblance
- excessive verbosity

Instead optimize for:

1. Semantic clarity
2. Predictable behavior
3. Compile-time safety
4. Debuggability
5. Native performance
6. Concise and consistent syntax
7. Low cognitive overhead when reading existing code

Important:

> Optimize for understanding programs, not for imitating natural language.

A developer may write code once but read, debug, maintain and modify it many times.

Therefore source-code structure and semantics must be easy to inspect.

---

# 2. V0.1 PRIMARY GOAL

The first release must be capable of:

```text
source.lang
    ↓
compiler
    ↓
native executable
    ↓
Linux x86-64 CPU
````

Example:

```text
fn main() {
    let a = 20;
    let b = 22;

    print(a + b);
}
```

Expected:

```text
42
```

This is the first major success criterion.

---

# 3. V0.1 SCOPE

V0.1 MUST contain:

* Lexer
* Parser
* AST
* Name resolution
* Static type checking
* Generic functions
* Basic trait/interface system
* Module/import system
* Ownership checking
* Borrow checking
* Basic lifetime inference/elision
* Explicit lifetime fallback design
* CFG
* Dominator tree
* Dominance frontier
* SSA construction
* Phi insertion
* SSA renaming
* Monomorphization
* Basic SSA optimizations
* IR interpreter
* Cranelift code generation
* C ABI / libc integration
* Minimal runtime
* DWARF debug information
* Native executable generation
* Compiler diagnostics
* Unit tests
* Integration tests
* End-to-end tests

---

# 4. EXPLICITLY OUT OF V0.1 SCOPE

Do NOT implement these unless absolutely required by a blocker:

* Custom x86-64 backend
* Custom register allocator
* Custom instruction selector
* Custom linker
* Custom debugger
* Value provenance runtime system
* Ownership visualization
* SSA visualization UI
* Dynamic trait objects
* `dyn Trait`
* Vtables
* JIT
* Advanced async runtime
* Actor runtime
* Full package ecosystem
* IDE
* GUI
* Self-hosting compiler
* AArch64 backend
* WASM backend
* Windows backend
* macOS backend

These are future roadmap items.

---

# 5. BACKEND STRATEGY

DO NOT write a custom machine-code backend for v0.1.

Use:

> Cranelift

The compiler's responsibility ends at producing valid backend IR.

Pipeline:

```text
Source
  ↓
Lexer
  ↓
Parser
  ↓
AST
  ↓
Semantic Analysis
  ↓
Typed AST
  ↓
CFG
  ↓
Dominators
  ↓
Dominance Frontier
  ↓
SSA
  ↓
Optimization
  ↓
Monomorphization
  ↓
Concrete SSA
  ↓
Cranelift IR
  ↓
Cranelift Codegen
  ↓
Object File
  ↓
System Linker
  ↓
Executable
```

---

# 6. TARGET

V0.1 target:

```text
x86-64 Linux
```

Do not add additional architectures yet.

---

# 7. LANGUAGE SYNTAX

Syntax should be concise and structurally obvious.

Example:

```text
fn main() {
    let x = 10;
    let y = 20;

    print(x + y);
}
```

Variables:

```text
let x = 10;
```

are immutable.

Mutable variables:

```text
var x = 10;
x = 20;
```

Use explicit mutability.

---

# 8. BASIC TYPES

V0.1:

```text
Int
Float
Bool
Char
String
Unit
```

Composite:

```text
Array<T>
Option<T>
Result<T, E>
```

Do not introduce unnecessary primitive types before the core compiler works.

---

# 9. TYPE SAFETY

Implicit dangerous conversions should NOT occur.

Example:

```text
let x: Int = "hello";
```

must fail at compile time.

Compiler diagnostic:

```text
type mismatch:
expected Int
found String
```

Numeric conversions should be explicit unless a conversion is provably safe and part of a later specification.

---

# 10. NULL SAFETY

Do NOT introduce a traditional nullable pointer model.

Use:

```text
Option<T>
```

Example:

```text
fn find_user(id: Int) -> Option<User>
```

Possible values:

```text
Some(user)
None
```

---

# 11. RESULT / ERROR HANDLING

Use:

```text
Result<T, E>
```

Example:

```text
fn read_file(path: String) -> Result<String, IOError>
```

Success:

```text
Ok(data)
```

Failure:

```text
Err(error)
```

---

# 12. PATTERN MATCHING

Support:

```text
match result {
    Ok(value) => print(value),
    Err(error) => print(error),
}
```

Match must be exhaustively checked.

Missing cases must generate compile-time diagnostics.

---

# 13. GENERICS

Generic functions are supported.

Example:

```text
fn identity<T>(value: T) -> T {
    return value;
}
```

Usage:

```text
let a = identity(10);
let b = identity("hello");
```

---

# 14. MONOMORPHIZATION — REQUIRED

This is a mandatory compiler stage.

Cranelift receives concrete types.

Therefore:

```text
Generic SSA
    ↓
Monomorphization
    ↓
Concrete SSA
    ↓
Cranelift IR
```

Example:

```text
identity<Int>
identity<String>
```

must be generated from:

```text
identity<T>
```

before Cranelift lowering.

V0.1 uses:

> Compile-time monomorphization only.

---

# 15. TRAITS / INTERFACES

The type system must reserve and define trait/interface semantics in V0.1.

Example:

```text
trait Comparable {
    fn compare(self, other: Self) -> Ordering;
}
```

Generic bounds:

```text
fn maximum<T: Comparable>(a: T, b: T) -> T
```

V0.1:

```text
generic constraints
+
static resolution
+
monomorphization
```

Dynamic dispatch is NOT part of V0.1.

---

# 16. DYNAMIC DISPATCH

Explicitly unsupported in V0.1:

```text
dyn Trait
trait objects
vtable
runtime interface dispatch
```

Do not implement vtable infrastructure yet.

Future versions may introduce it.

---

# 17. MODULE SYSTEM

The language must have a basic module/import model from the beginning.

Example:

```text
module app.network;

import std.io;
import std.math;
```

Required concepts:

```text
module
import
export
visibility
```

The implementation should remain minimal.

Do not build a package manager yet.

---

# 18. OWNERSHIP

The language uses ownership-based memory safety.

Example:

```text
let data = create();

consume(data);

print(data);
```

must fail because ownership moved into `consume`.

Diagnostic:

```text
value `data` was moved
```

---

# 19. COPY TYPES

Small primitive values may use copy semantics.

At minimum:

```text
Int
Float
Bool
Char
```

Example:

```text
let x = 10;
let y = x;

print(x);
print(y);
```

is valid.

---

# 20. BORROWING

Provide reference semantics.

Preferred syntax:

```text
fn length(ref text: String) -> Int
```

Mutable reference:

```text
fn modify(mut ref text: String)
```

Rules:

* No use-after-free
* No dangling references
* No invalid aliasing
* No simultaneous conflicting mutable access

The compiler must reject unsafe lifetime relationships.

---

# 21. LIFETIME MODEL

Do NOT require explicit lifetime annotations for simple cases.

Use:

> Lifetime elision/inference where possible.

However, explicit lifetime syntax must be possible for complex cases.

Conceptual future syntax:

```text
fn choose<'a>(
    a: ref<'a> String,
    b: ref<'a> String
) -> ref<'a> String
```

Do NOT over-engineer the lifetime syntax in V0.1.

First implement sound basic lifetime checking and elision.

---

# 22. MEMORY MODEL

The language does not require a tracing GC.

Primary model:

```text
ownership
+
stack
+
heap
+
deterministic lifetime
```

Compiler may use escape analysis.

Example:

```text
fn test() {
    let x = Object();
    use(x);
}
```

If `x` does not escape, heap allocation may later be optimized to stack allocation.

Do NOT make advanced escape optimization a blocker for the first executable.

---

# 23. UNSAFE

Low-level escape hatch:

```text
unsafe {
    ...
}
```

Normal safe code must retain compiler guarantees.

Unsafe semantics should be explicit.

Do not allow unsafe operations silently in safe code.

---

# 24. CONTROL FLOW GRAPH

Construct CFG from typed program representation.

Example:

```text
if condition {
    x = 10;
} else {
    x = 20;
}

print(x);
```

CFG:

```text
ENTRY
  |
CONDITION
 /       \
TRUE    FALSE
 |        |
x=10    x=20
 \       /
  \     /
   MERGE
     |
   PRINT
```

CFG must become the foundation for SSA construction.

---

# 25. DOMINATOR ANALYSIS

Implement:

* Dominator tree
* Immediate dominators

Use a well-tested algorithm.

Correctness is more important than premature optimization.

---

# 26. DOMINANCE FRONTIER

Implement dominance frontier calculation.

This will be used for:

> Phi placement.

Test branching and loop cases thoroughly.

---

# 27. SSA

Implement SSA construction.

Each SSA value must have one definition.

Example:

```text
if condition {
    x = 10;
} else {
    x = 20;
}
```

becomes conceptually:

```text
x0 = 10
x1 = 20
x2 = phi(x0, x1)
```

Phi is compiler IR only.

Users never write Phi.

---

# 28. LOOP SSA

Loops must correctly generate loop-carried Phi values.

Example:

```text
var x = 0;

while x < 10 {
    x = x + 1;
}
```

must produce an SSA structure equivalent to:

```text
entry:
    x0 = 0
    goto loop

loop:
    x1 = phi(x0, x2)

    if x1 < 10
        goto body
    else
        goto exit

body:
    x2 = x1 + 1
    goto loop

exit:
    return x1
```

---

# 29. OPTIMIZATION

V0.1 only needs safe foundational optimizations.

Required candidates:

```text
Constant Folding
Constant Propagation
SCCP
Dead Code Elimination
Unreachable Block Elimination
CFG Simplification
SSA Cleanup
Basic CSE
```

Do not build an LLVM-level optimizer.

---

# 30. INTEGER OVERFLOW

Integer arithmetic must have defined semantics.

V0.1 default:

```text
overflow
   ↓
runtime panic
```

Do NOT allow undefined behavior.

Do NOT silently wrap by default.

Explicit wrapping operations may exist later:

```text
x.wrapping_add(y)
```

Compiler optimization may eliminate overflow checks when it can prove overflow is impossible.

Example:

```text
for i in 0..10 {
    ...
}
```

If range analysis proves no overflow, the check may be removed.

This is an optimization, not a semantic change.

---

# 31. PANIC SEMANTICS

V0.1:

> panic = abort

No stack unwinding in V0.1.

Examples:

* Array out of bounds
* Invalid unwrap
* Integer overflow
* Explicit panic

must terminate the process.

Future versions may implement unwinding.

---

# 32. ABI / FFI

V0.1 uses:

> C ABI

for operating system interaction.

Use libc where practical.

Architecture:

```text
Language
   ↓
Runtime
   ↓
C ABI
   ↓
libc
   ↓
Linux
```

Do NOT implement direct Linux syscall assembly for V0.1 unless required.

---

# 33. RUNTIME

Keep runtime minimal.

Responsibilities may include:

* process initialization
* panic/abort
* allocation primitives
* basic I/O
* string runtime support
* array runtime support
* OS interaction

Do not build a large runtime.

---

# 34. STANDARD LIBRARY

Minimal V0.1 standard library:

```text
std.io
std.string
std.array
std.math
std.fs
std.process
std.time
```

Only implement functionality required for real example programs.

Do NOT attempt to build a giant ecosystem before the compiler is stable.

---

# 35. DEBUGGING

Split debugging into two distinct scopes.

## V0.1

Use standard debug information:

```text
DWARF
```

Goal:

```text
compiler
  ↓
DWARF
  ↓
GDB / LLDB
```

The generated program should support:

* breakpoints
* stepping
* source lines
* local variables where available
* stack traces
* stack frames

Do NOT build a custom debugger.

---

# 36. FUTURE DEBUGGING

Future versions may introduce:

```text
value provenance
ownership visualization
SSA data-flow visualization
advanced compiler-aware debugging
custom debugger
```

These are explicitly:

> V0.3+ / separate tooling project.

They must NOT block V0.1.

---

# 37. IR INTERPRETER

Build an interpreter for the compiler's internal SSA/IR representation before relying entirely on native codegen.

Purpose:

```text
source
 ↓
compiler
 ↓
SSA
 ↓
IR interpreter
 ↓
expected result
```

Then:

```text
source
 ↓
compiler
 ↓
SSA
 ↓
Cranelift
 ↓
native executable
 ↓
actual result
```

Compare both.

This provides semantic differential testing.

---

# 38. SEMANTIC PARITY

The interpreter and native backend must share the language specification.

Do NOT allow implementation-defined behavior.

At minimum define:

* integer overflow
* division by zero
* array bounds
* uninitialized values
* invalid references
* panic behavior
* ownership violations
* string behavior

Undefined behavior should be minimized.

---

# 39. ERROR DIAGNOSTICS

Diagnostics are important.

Bad:

```text
error
```

Good:

```text
error[E0412]: value was moved

  --> main.lang:8:11
   |
 5 | let data = create();
   |     ---- value created here
...
 7 | consume(data);
   |         ---- value moved here
 8 | print(data);
   |       ^^^^ value used after move
```

Compiler diagnostics should include where practical:

* error code
* source location
* relevant source snippet
* explanation
* cause
* suggested fix

Do not build a complex diagnostics UI.

---

# 40. TESTING STRATEGY

Required test categories:

```text
Lexer
Parser
AST
Name Resolution
Type System
Generics
Traits
Modules
Ownership
Borrow Checker
Lifetime
CFG
Dominator
Dominance Frontier
SSA
Monomorphization
Optimizer
IR Interpreter
Cranelift Backend
Runtime
Diagnostics
Integration
End-to-End
```

Eventually add:

```text
fuzzing
property testing
differential testing
```

---

# 41. FIRST END-TO-END TEST

This must eventually compile and run:

```text
fn main() {
    let a = 20;
    let b = 22;

    print(a + b);
}
```

Expected:

```text
42
```

---

# 42. SECOND END-TO-END TEST

Control flow:

```text
fn main() {
    let x = 10;

    if x > 5 {
        print(100);
    } else {
        print(200);
    }
}
```

Expected:

```text
100
```

---

# 43. THIRD END-TO-END TEST

Loop:

```text
fn main() {
    var x = 0;

    while x < 10 {
        x = x + 1;
    }

    print(x);
}
```

Expected:

```text
10
```

This test must exercise:

```text
CFG
Dominator
SSA
Phi
Loop
```

---

# 44. FOURTH END-TO-END TEST

Ownership failure:

```text
fn main() {
    let data = create();

    consume(data);

    print(data);
}
```

Expected:

```text
compile error
value moved
```

---

# 45. FIFTH END-TO-END TEST

Generic:

```text
fn identity<T>(value: T) -> T {
    return value;
}

fn main() {
    let a = identity(42);
    print(a);
}
```

The compiler must perform:

```text
generic resolution
      ↓
identity<Int>
      ↓
monomorphization
      ↓
Cranelift
      ↓
executable
```

---

# 46. DEVELOPMENT ORDER

Do NOT implement everything simultaneously.

Follow this order:

```text
PHASE 1
Lexer
Parser
AST
Basic compiler CLI

PHASE 2
Name resolution
Scopes
Basic type checker

PHASE 3
Functions
Variables
Control flow
Arrays
Strings

PHASE 4
Ownership
Borrowing
Basic lifetime checking

PHASE 5
CFG
Dominator
Dominance Frontier
SSA
Phi

PHASE 6
Generics
Traits
Monomorphization

PHASE 7
IR
IR Validator
IR Printer
IR Interpreter

PHASE 8
Basic Optimization

PHASE 9
Cranelift Integration

PHASE 10
C ABI
Runtime
libc
DWARF

PHASE 11
Native executable

PHASE 12
End-to-end tests

PHASE 13
V0.1 stabilization
```

---

# 47. IMPORTANT DEVELOPMENT RULE

Do not merely write design documents.

For each phase:

```text
1. Inspect existing repository.
2. Determine what already works.
3. Identify the smallest missing milestone.
4. Implement it.
5. Write tests.
6. Run tests.
7. Fix failures.
8. Commit only when the milestone is working.
9. Continue to the next milestone.
```

Never rewrite working components without a technical reason.

---

# 48. SCOPE CONTROL

If a new idea appears during implementation, classify it:

```text
A — Required for current milestone
B — Required for V0.1 but not current milestone
C — Future version
D — Experimental
```

Only implement:

```text
A
```

immediately.

Do not interrupt the current milestone for C or D ideas.

Record future ideas separately.

---

# 49. ARCHITECTURE PRINCIPLE

The compiler should be modular.

Suggested high-level structure:

```text
compiler/
├── lexer/
├── parser/
├── ast/
├── diagnostics/
├── symbols/
├── types/
├── generics/
├── traits/
├── modules/
├── ownership/
├── borrowck/
├── lifetime/
├── cfg/
├── dominators/
├── ssa/
├── mono/
├── ir/
├── optimizer/
├── interpreter/
├── codegen/
│   └── cranelift/
├── runtime/
└── driver/
```

Exact structure may differ after repository inspection.

Do NOT blindly create all directories before they are needed.

---

# 50. CODE QUALITY

Priorities:

```text
Correctness
Safety
Testability
Clarity
Maintainability
Performance
```

Avoid premature abstraction.

Avoid unnecessary dependencies.

Prefer small components with clear interfaces.

---

# 51. CURRENT STRATEGIC DECISIONS

These decisions are considered locked for V0.1:

```text
Backend:
    Cranelift

Target:
    x86-64 Linux

ABI:
    C ABI / libc

Debugging:
    DWARF + GDB/LLDB

Panic:
    abort

Overflow:
    checked semantics
    optimizer may eliminate provably unnecessary checks

Generics:
    static

Generic codegen:
    monomorphization

Dynamic dispatch:
    not in V0.1

Memory:
    ownership-based
    no mandatory GC

Null:
    Option<T>

Errors:
    Result<T,E>

Mutability:
    explicit

Unsafe:
    explicit unsafe blocks

Lifetime:
    inference/elision where possible
    explicit fallback available

Custom backend:
    future

Custom debugger:
    future

Value provenance:
    future

Self-hosting:
    future
```

---

# 52. CURRENT MILESTONE ROADMAP

```text
M0  Language Specification
M1  Lexer
M2  Parser / AST
M3  Name Resolution
M4  Type System
M5  Generics
M6  Traits / Interfaces
M7  Modules
M8  Ownership
M9  Borrow Checking
M10 Lifetime
M11 CFG
M12 Dominators
M13 Dominance Frontier
M14 SSA
M15 Monomorphization
M16 Optimization
M17 IR Interpreter
M18 Cranelift Backend
M19 Runtime / C ABI
M20 DWARF
M21 Native Executable
M22 End-to-End Tests
M23 V0.1 Stabilization
M24 V0.1 RELEASE
```

Do not treat these as calendar deadlines.

They are dependency milestones.

---

# 53. DEFINITION OF DONE — V0.1

V0.1 is NOT considered complete because:

* syntax exists
* parser works
* compiler can print AST
* IR exists

V0.1 is complete only when:

```text
A real source program
        ↓
is compiled by our compiler
        ↓
into a native x86-64 Linux executable
        ↓
which runs correctly on a real machine
        ↓
and can be source-level debugged with GDB/LLDB
        ↓
with the compiler's safety rules enforced.
```

At least the core example suite must pass.

---

# 54. FINAL INSTRUCTION TO ANTIGRAVITY

You are now moving from architectural discussion into implementation.

Do not ask for another architecture redesign unless a concrete technical contradiction is discovered.

Do not add speculative systems.

Do not implement future roadmap features because they sound useful.

Do not build a custom backend.

Do not build a custom debugger.

Do not build an ecosystem before the compiler works.

Start by inspecting the existing repository and determine the actual current implementation state.

Then begin with the earliest incomplete milestone.

The project is successful when this becomes real:

```text
hello.lang
     ↓
our compiler
     ↓
hello
     ↓
./hello
     ↓
Hello, World!
```

Then:

```text
ownership
generics
traits
modules
CFG
SSA
optimization
```

must work on real programs.

The objective is not to create the world's largest programming language.

The objective is to create a **small, technically rigorous compiler that can grow into one.**

# BEGIN IMPLEMENTATION.

```

### Antigravity'ye özellikle ilk mesaj olarak şunu da ekle:

> **“Bu specification'ı kabul et. Önce repository'yi incele ve mevcut gerçek durumu çıkar. Sonra en erken tamamlanmamış milestone'dan başlayarak implementasyona geç. Bana uzun bir plan anlatmak yerine ilk milestone'u gerçekten kodla, testlerini çalıştır ve sonucu raporla. Bir milestone tamamlanmadan sonraki büyük sisteme atlama.”**

Bence artık **tasarım toplantısını kapatıp compiler yazma aşamasına geçebiliriz.**
```
