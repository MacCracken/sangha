# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x     | Yes       |
| < 1.0   | No        |

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please report it
responsibly through **GitHub Security Advisories**:

1. Go to the [Security tab](../../security/advisories) of this repository.
2. Click **"Report a vulnerability"**.
3. Fill in the details and submit.

**Do not open a public issue for security vulnerabilities.**

## Response Timeline

| Action | Target |
|---|---|
| Acknowledgement | Within **48 hours** |
| Initial assessment | Within **5 business days** |
| Fix for critical severity | Within **14 days** |
| Fix for high severity | Within **30 days** |
| Fix for moderate/low severity | Next scheduled release |

## Scope

This policy covers the published API. Vulnerabilities in dependencies should
be reported to the respective upstream projects (and flagged here if they
affect users).

## Design Principles

Sangha follows these security-relevant design principles:

- **Zero panics**: No `unwrap()`, `expect()`, or unchecked indexing in library code. All operations return `Result`.
- **Input validation at boundaries**: Every public function validates parameters before computation. NaN, infinity, negative values, and out-of-bounds indices are rejected.
- **No I/O**: Pure computation library with no file, network, or system access.
- **Minimal dependencies**: Only `serde`, `thiserror`, and `tracing` as required dependencies. Optional dependencies are feature-gated.
- **Supply chain**: `cargo-audit` (advisory database), `cargo-deny` (license + ban + source verification), and CI-enforced security scanning on every push.
- **Checked arithmetic**: Floating-point validation via `validate_finite`, `validate_positive`, `validate_non_negative` helpers. No silent NaN propagation.
- **Serde safety**: Types with invariants provide `validate()` methods for post-deserialization checking. Consumers should call `validate()` after deserializing untrusted data.

## Disclosure

We follow coordinated disclosure. Once a fix is released, we will publish a
security advisory crediting the reporter (unless anonymity is requested).
