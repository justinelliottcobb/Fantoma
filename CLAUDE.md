# Fantoma

## Coding Guidelines

1. **Idiomatic Rust** — Follow Rust conventions: prefer iterators over loops, use `Result`/`Option` instead of sentinel values, leverage pattern matching, prefer ownership over borrowing where ergonomic, use `impl Trait` for return types.
2. **Test-Driven Development** — Write tests first, then implement. Every public function must have tests. Use `#[cfg(test)]` modules in each source file plus integration tests in `tests/`.
3. **Phantom Types & Algebraic Patterns** — Use phantom types for type-level state machines and compile-time guarantees. Prefer sum types (enums) over boolean flags or stringly-typed data. Use the newtype pattern for domain-specific wrappers.
4. **CPU Concurrency via Rayon** — Use `rayon` for data-parallel workloads. Prefer `.par_iter()` over manual thread management.
5. **GPU Acceleration via wgpu** — Use `wgpu` for GPU-accelerated compute. Abstract GPU operations behind traits to allow fallback to CPU paths.

## Linting & Formatting

- `cargo fmt` — enforced on commit and CI
- `clippy` with `deny(clippy::all)`, `warn(clippy::pedantic, clippy::nursery)` — enforced on commit and CI
- `unsafe` code is forbidden

## Git Workflow

- **Gitflow-like**: `feature/*`, `chore/*`, `fix/*` branches → PR to `develop` → release PR → `main`
- Pre-commit hook runs `cargo fmt --check`, `cargo clippy`, and `cargo test`
- CI runs the same checks on every PR push

## Commands

- `cargo fmt` — format code
- `cargo clippy -- -D warnings` — lint
- `cargo test` — run all tests
- `cargo build` — build debug
- `cargo build --release` — build release
