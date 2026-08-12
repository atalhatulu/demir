# Future Type System: Typed AI Computation

## 1. Overview
In standard system languages, computation revolves around scalars, arrays, and structs. In our AI-First language, the type system will natively treat AI and scientific computation boundaries as first-class, verifiable entities.

**Status:** Research & Vision (V2+). *Not to be implemented in V0.1.*

## 2. Dimensional and Probabilistic Typing

### Tensor & Matrix Types
Instead of treating machine learning structures as generic heap-allocated arrays, the type system will natively understand dimensions and layout.

```rust
// Future Vision
Tensor<f32, [1, 768]>
Tensor<f32, [224, 224, 3]>
Vector<f32, 768>
```

### Compile-Time Shape Verification
The compiler must statically catch dimension/type mismatches before runtime.
For example, attempting to pass a `Tensor<f32, [1, 512]>` into a function expecting `Tensor<f32, [1, 768]>` will yield a strict compile-time type error rather than a runtime shape exception.

```rust
// Future Vision
fn classify(
    input: Tensor<f32, [1, 768]>
) -> Distribution<Label>
```

### Probability & Distribution
Standard booleans and floats do not capture AI output accurately. The type system will include primitives representing uncertainty.
- `Probability`
- `Distribution<T>`
- `Fuzzy<T>`

## 3. Hardware / Backend Lowering
By elevating Tensors and Distributions to the compiler's type checker, our IR (MIR/SSA) can lower these types directly into specialized hardware backends in the future.
- **CPU:** Standard vectorized (SIMD) implementations.
- **GPU / NPU / Accelerators:** Direct PTX/SPIR-V or accelerator-specific code generation.

**Current Rule:** In V0.1, we do **not** write GPU/NPU backends. We compile strictly to CPU native machine code via Cranelift. The type system's foundation is merely designed with this future in mind.
