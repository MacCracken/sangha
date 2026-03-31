# Contributing to Sangha

Thank you for considering a contribution! This document covers the development workflow, coding standards, testing, and review process.

## Prerequisites

- Rust stable (MSRV: 1.89)
- `cargo-llvm-cov` for coverage: `cargo install cargo-llvm-cov`
- `cargo-audit` for security: `cargo install cargo-audit`
- `cargo-deny` for supply chain: `cargo install cargo-deny`

## Development Setup

```bash
git clone https://github.com/MacCracken/sangha.git
cd sangha
rustup show
make check
```

## Makefile Targets

| Target | Description |
|--------|-------------|
| `make check` | Full lint + test + audit (default) |
| `make fmt` | Check formatting |
| `make clippy` | Lint with `-D warnings` |
| `make test` | Run all tests (`--all-features` + `--no-default-features`) |
| `make audit` | Security advisory scan |
| `make deny` | License + supply chain check |
| `make bench` | Run benchmarks, update `bench-history.csv` and `benchmarks.md` |
| `make coverage` | Generate HTML coverage report |
| `make coverage-check` | Enforce 70% minimum coverage |
| `make doc` | Build docs with `-D warnings` |
| `make build` | Release build |
| `make clean` | Clean build artifacts |

## Git Workflow

- Branch from `main`
- One logical change per commit
- Use conventional commit messages: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `ci:`
- PR against `main` with a clear description

## Adding a New Module

1. Create `src/{module}.rs` following the flat module pattern
2. Add `pub mod {module};` to `src/lib.rs` (alphabetical order)
3. Implement types + functions + unit tests
4. Run `make check` (fmt, clippy, test, audit)
5. Add benchmarks to `benches/benchmarks.rs`
6. Add 1-2 integration tests to `tests/integration.rs`
7. Run `make bench` to record baselines
8. Update `docs/architecture/overview.md` module map

## Code Conventions

- **`#[non_exhaustive]`** on all public enums and structs
- **`#[must_use]`** on all pure functions
- **`#[inline]`** on short hot-path functions (single formula)
- **`Serialize + Deserialize`** on all types (serde)
- **Serde roundtrip test** for every serializable type
- **`validate()`** method on types with fallible constructors
- **Zero `unwrap`/`panic`** in library code
- **`Result<T, SanghaError>`** for all fallible operations
- **Validation at entry**: use `validate_finite`, `validate_positive`, `validate_non_negative`
- **Doc comments** on all public items with `# Errors` section on fallible functions

## Testing

```bash
make test              # all features + no-default-features
make bench             # criterion benchmarks with history tracking
make coverage          # HTML coverage report
make coverage-check    # enforce 70% floor
```

Every new function needs:
- Happy-path test with known reference values
- Edge-case tests (empty input, zero, single element, boundary)
- Error-path tests (invalid parameters, NaN, out of bounds)
- Serde roundtrip test for any new types

## Benchmarks

Every computationally meaningful function needs a criterion benchmark. Run `make bench` before and after changes to detect regressions. The script tracks history in `bench-history.csv` and generates `benchmarks.md`.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only (see [LICENSE](LICENSE)).
