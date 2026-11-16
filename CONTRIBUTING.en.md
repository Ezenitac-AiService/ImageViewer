# Contributing to ImageViewer (`Ezenitac-AiService/ImageViewer`)

**Language / 언어**: [한국어](CONTRIBUTING.md) | **[English](CONTRIBUTING.en.md)**

Thank you for contributing to **ImageViewer**!

## Independent Fork Workflow

`Ezenitac-AiService/ImageViewer` is an **independent cross-platform Rust fork** of `edith2580/ImageViewer` (`v1.0.6`).

- **No Upstream Merges**: Do **not** merge commits or pull requests from `edith2580/ImageViewer` (`upstream`). The C#/.NET `v1.0.6` baseline is archived at tag `legacy-v1.0.6`, and active development on `main` is pure Rust.
- **License Compatibility**: All contributions must be compatible with `GPL-3.0-only`. Any new Cargo dependency must pass `deny.toml` (`cargo deny check licenses`) and be documented in `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_NOTICES.en.md` if user-facing/core.
- **Issue Routing**: File issues and PRs strictly against `Ezenitac-AiService/ImageViewer`. Never redirect fork-specific issues to `edith2580/ImageViewer`.

## Development & Quality Gates

Before opening a pull request, verify all workspace checks pass locally:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Coding Standards
- Follow Test-Driven Development (write failing tests before production fixes).
- Preserve Zero-Trust path, archive, and HTTPS update invariants (no `http://` or test-only security bypasses).
- Keep user-facing commit summaries in Korean per project governance (`.specify/memory/constitution.md` Principle I), and code/comments/specs in English (Principle II), with Korean default (`*.md`) and English counterpart (`*.en.md`) root documentation.
