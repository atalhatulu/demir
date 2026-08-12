# AI-First Design Principles

## Overview
This language is engineered as an **AI-First System Programming Language**. However, this does not mean it uses natural language (e.g., English sentences) as syntax. 

Instead, it provides an environment where **AI models can predictably generate structurally sound, deterministic, and verifiable code**, while the compiler strictly enforces system-level safety and generates native AOT binaries.

## 1. Intent vs Implementation
- **AI generates intent:** The LLM maps human requests into strict language abstractions (Capabilities, Intents, Contracts).
- **Compiler verifies and lowers:** The compiler does not "guess" meaning. It statically verifies the AI's intent against type systems, ownership rules, and data provenance, rejecting hallucinations at compile-time.

## 2. Syntax Design Priorities
When choosing syntax and structural rules, the language strictly adheres to the following priority hierarchy:
1. **Semantic Clarity:** The meaning of a construct must be unambiguous.
2. **Compiler Verifiability:** The compiler must be able to mathematically prove memory safety and contract adherence.
3. **AI Code Generation Reliability:** The syntax must be easy for an LLM to emit consistently without syntax errors or nested complexity failures.
4. **Memory Safety:** Borrow checking and data race prevention.
5. **Native Performance:** Zero-cost abstractions leading to fast Cranelift/LLVM lowered instructions.
6. **Human Readability:** It is the *least* prioritized factor if it conflicts with any of the above. We will not sacrifice verifiability or AI reliability to make the language "look like Python."

## 3. Disallowed Anti-Patterns (V0.1 and Beyond)
- **Prompt Compilers:** We are not building a parser that takes free-text "make a server" and dynamically compiles it.
- **Natural Language Parsing:** Language syntax must be strict and tokenizable, not fuzzy text processing.
- **LLM Runtimes in V0.1:** Do not inject LLM inference engines into the generated binaries. V0.1 produces pure, deterministic, and isolated Linux ELF executables.
