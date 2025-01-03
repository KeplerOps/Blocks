# API Stability Policy [DRAFT]

## Core Principles

1. **Semantic Versioning**
   - MAJOR: Breaking changes with proof of necessity
   - MINOR: Features with proof of non-interference
   - PATCH: Bug fixes with verification

2. **Stability Guarantees**
   - Public API marked with stability attributes
   - Breaking changes require formal proof
   - Deprecation cycle enforced by compiler

3. **Version Compatibility**
   - Explicit compatibility bounds
   - Feature flag version requirements
   - Cross-crate version constraints

## API Annotations

### Stability Markers

```rust
#[api(stable = "1.0.0")]
pub trait Verified {
    fn verify(&self) -> Proof;
}

#[api(beta = "1.1.0")]
pub trait VerifiedParallel: Verified {
    fn verify_parallel(&self) -> Proof;
}

#[api(experimental)]
pub trait VerifiedGPU: Verified {
    fn verify_gpu(&self) -> Proof;
}
```

### Breaking Change Markers

```rust
#[api(breaking = "2.0.0", reason = "Performance optimization")]
#[verify(safety = "proven")]
pub fn optimized_sort<T: Verified>(data: &mut [T])

#[api(deprecated = "1.1.0", replacement = "optimized_sort")]
pub fn old_sort<T>(data: &mut [T])
```

## Version Rules

### Public API

1. Stable APIs:
   - Must have formal verification
   - Cannot change without major version
   - Must maintain proof compatibility

2. Beta APIs:
   - Must have initial verification
   - May change with minor version
   - Must document instability

3. Experimental APIs:
   - Must be feature gated
   - No stability guarantees
   - Must not break stable APIs

### Breaking Changes

1. Requirements:
   - Formal proof of necessity
   - Migration documentation
   - Verification of all impacts
   - Performance analysis

2. Process:
   ```rust
   // Step 1: Mark for deprecation
   #[api(deprecating = "1.1.0")]
   pub fn old_api() { }

   // Step 2: Add replacement
   #[api(stable = "1.1.0")]
   pub fn new_api() { }

   // Step 3: Remove in next major
   #[api(removed = "2.0.0")]
   pub fn old_api() { }
   ```

## Verification Requirements

### API Changes

1. Proof Requirements:
   ```rust
   #[verify_api_change]
   mod api_v2 {
       #[prove(compatibility)]
       fn verify_breaking_change() -> Proof;

       #[prove(performance)]
       fn verify_performance_impact() -> Proof;
   }
   ```

2. Migration Verification:
   ```rust
   #[verify_migration("1.0.0", "2.0.0")]
   mod migration {
       #[prove(correctness)]
       fn verify_migration_path() -> Proof;
   }
   ```

## Tooling

### Version Checker

```rust
#[derive(ApiVersion)]
#[api_version(current = "1.0.0")]
pub struct Api {
    #[verify_compatibility(since = "0.1.0")]
    pub fn stable_api(&self);

    #[verify_compatibility(breaking = "2.0.0")]
    pub fn breaking_api(&self);
}
```

### Compatibility Matrix

Generated in CI:
```
| API              | Since | Stable | Breaking |
|-----------------|--------|---------|-----------|
| stable_api      | 0.1.0  | 1.0.0   | None      |
| breaking_api    | 1.0.0  | 1.0.0   | 2.0.0     |
```

## Implementation Guidelines

1. New APIs:
   ```rust
   #[api(experimental)]
   #[verify(safety = "proven")]
   mod new_feature {
       // Implementation with proofs
   }
   ```

2. API Evolution:
   ```rust
   #[api(evolving = "1.1.0")]
   #[verify(compatibility = "1.0.0")]
   mod evolving_api {
       // Backwards compatible changes
   }
   ```

## Continuous Integration

1. Version Verification:
   - API compatibility checks
   - Breaking change detection
   - Proof regression testing

2. Documentation:
   - API compatibility tables
   - Migration guides
   - Breaking change logs

## Error Messages

Clear error messages for version violations:
```
error[E0123]: breaking change in stable API
  --> src/lib.rs:10:1
   |
10 | pub fn stable_api() {
   | ^^^^^^^^^^^^^^^^^^^ breaking change in API marked stable since 1.0.0
   |
   = note: API changes must be marked with #[api(breaking = "2.0.0")]
   = help: consider adding a replacement API first
``` 