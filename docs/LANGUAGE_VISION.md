# Language Vision: AI-First System Programming Language

## 1. Core Philosophy
The programming language developed in this project aims to create a hybrid that brings together:
- Low-level control of C
- Memory safety approach of Rust
- Modern type system capabilities
- Native compilation and performance
- **High manufacturability/generation by AI models**

### 1.1 The Meaning of "AI-First"
"AI-first" does **not** mean interpreting natural language sentences as code. 
Instead, it means designing a highly structured, strictly typed language where:
1. **AI models generate intent:** AI outputs highly deterministic and verifiable language constructs.
2. **Compiler verifies intent:** The compiler formally verifies the types, lifetime, contracts, and architecture.
3. **Compiler generates native code:** Downstream processing results in highly optimized, native AOT machine code via our strict SSA pipeline.

Our primary criteria for syntax and language features:
`Semantic Clarity > Compiler Verifiability > AI Code Generation Reliability > Memory Safety > Native Performance > Human Readability`

While human readability is important, we do not compromise compiler semantics or performance just to make syntax mimic human/scripting languages (like Python) if it makes deterministic generation by an LLM harder.

## 2. Project Roadmap and Milestones

### V0.1 - The Compiler Backbone (Current Phase)
*Status: Active / Implementation*
- Rust bootstrap compiler (`rustc`).
- Traditional procedural semantics (`fn main()`, primitive types, `if/while`).
- Advanced internal representation: AST -> HIR -> MIR -> CFG -> Strict SSA.
- Native code generation via Cranelift.
- JIT / AOT (ELF executable) compilation.
- **Rule:** Do not force AI-first features, formal verifiers, or GPU backends into V0.1.

### V0.2 - V1.0 - Language Maturity
*Status: Future*
- Robust Ownership/Borrowing implementation in the compiler.
- Advanced primitive and compound types (Structs/Enums).
- Provenance/Data Lineage tracking metadata at the SSA level.

### V1+ / V2+ - AI-First Paradigms
*Status: Research & Future Implementation*
- Capabilities and Agent-Oriented Programming (AOP).
- Intent-Oriented structures natively recognized by the compiler.
- Contract-Oriented constraints (requires, ensures, invariant).
- Hardware-specific backends (GPU/NPU) and Typed AI Computations (Tensors, Probability).
- Self-hosted compiler.
