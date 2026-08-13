# Language Paradigms

This document outlines the core paradigms that will define the AI-First System Programming Language. 

**IMPORTANT NOTE:** Most of these paradigms (Capabilities, Agents, Intents) are **V0.3+ / V2+ Vision** and must NOT be forced into the V0.1 implementation, which focuses on stable, procedural SSA compilation. The one exception is **Section 3 (Contract / Proof-Oriented Programming)**: `requires` / `ensures` are already implemented and enforced at runtime in V0.1.

## 1. Capability / Agent-Oriented Programming (AOP)
Moving away from classic OOP (classes, inheritance, objects), the language will embrace a capability and agent-centric architecture.

### Concept
An `agent` is an isolated computational unit consisting of:
- `state`
- `capabilities`
- `permissions`
- `typed inputs / outputs`

```rust
// Future Paradigm (V2+)
agent PaymentProcessor {
    state {
        balance: Money
    }

    capability charge(
        user: User,
        amount: Money
    ) -> Result<Transaction>
}
```
Capabilities are invoked using specific semantic operations (e.g., `ask payment.charge(user, amount)`), which the compiler will ultimately lower into traditional SSA function calls and ownership boundaries.

## 2. Intent-Oriented Programming
Instead of relying on fragile natural language interpretation at the compiler level, the language will provide a **deterministic intent syntax**. LLMs will generate this syntax, which the compiler will strictly translate to deterministic IRs.

```rust
// Future Paradigm (V2+)
intent users =
    select User
    where User.age >= 18
    order User.name
```
**Pipeline:** `Intent -> Typed semantic representation -> HIR -> MIR -> CFG -> SSA`

## 3. Contract / Proof-Oriented Programming
Functions move beyond just input/output signatures to include verifiable behavioral contracts.

**Current status (V0.1):** `requires` pre-conditions and `ensures` post-conditions are **implemented** and enforced at runtime — a violated pre-condition aborts with `ASSERT FAILED: pre-condition N failed` and a non-zero exit status. This is the first concrete, working piece of the contract model.

```rust
// Implemented today (V0.1)
fn divide(a: Int, b: Int) -> Int
    requires b != 0
{
    return a / b;
}

fn sort(data: Slice<Int>) -> Slice<Int>
    ensures ordered(result)
    ensures same_elements(data, result)
{
    // ...
}
```

**Categories:**
- `requires` (Pre-conditions) — implemented (runtime)
- `ensures` (Post-conditions) — implemented (runtime)
- `invariant` (State/Loop consistency) — future

**Compiler Verification (Future, V0.3+):**
The compiler architecture (specifically SSA/CFG metadata) is built to eventually support statically proving these constraints, generating runtime checks, or rejecting unverified logic. Today the checks are runtime-only.

## 4. Unifying the Paradigms
These paradigms are not disjoint syntax features; they are vertically integrated.

```text
AI-FIRST SYSTEM LANGUAGE
                ┌───────────────┐
                │  CAPABILITY   │
                └───────┬───────┘
                        │
                ┌───────▼───────┐
                │    INTENT     │
                └───────┬───────┘
                        │
                ┌───────▼───────┐
                │   CONTRACT    │
                └───────┬───────┘
                        │
                ┌───────▼───────┐
                │  TYPED VALUE  │
                └───────┬───────┘
                        │
                ┌───────▼───────┐
                │  PROVENANCE   │
                └───────┬───────┘
                        │
                        ▼
                       HIR / MIR / SSA -> Native
```
