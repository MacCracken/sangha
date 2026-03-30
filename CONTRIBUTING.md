# Contributing to sangha

Thank you for considering a contribution! This document covers the development
workflow, coding standards, and review process.

## Development Setup

```bash
git clone https://github.com/MacCracken/sangha.git
cd sangha
rustup show
make check
```

## Pull Request Process

1. **Fork and branch** from `main`.
2. **Keep commits focused** — one logical change per commit.
3. **Write tests** — new features require tests; bug fixes require regression tests.
4. **Run CI locally** before pushing: `make check`
5. **Open a PR** against `main` with a clear description.
6. **Address review feedback**.

## Code Style

- Follow `rustfmt` defaults (enforced by CI).
- Zero clippy warnings.
- Public API items must have doc comments.
- Use `#[inline]` on small, hot-path functions.
- Use `#[non_exhaustive]` on public enums.

## Testing

```bash
cargo test --all-features
make bench
make coverage
```

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0 (see [LICENSE](LICENSE)).
