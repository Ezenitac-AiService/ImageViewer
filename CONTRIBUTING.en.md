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
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
cargo test --workspace
cargo deny check advisories bans licenses sources
cargo deny --all-features check advisories bans licenses sources
packaging/verify/component-count.sh
```

Both the shipped configuration (default features) and the all-features configuration are tested. All features include work that is not shipped yet (updater, OS codecs, extended formats, shell registration); see the [capability register](docs/CAPABILITIES.en.md).

## Definition of Done

A task is done only when:

1. A failing test was written first and now passes.
2. The behaviour was **observed in the running application** on each supported OS (Windows, macOS), or the PR states which OS was not checked. Passing library tests alone is not done.
3. Any capability newly visible in the app or the docs is listed in the [capability register](docs/CAPABILITIES.en.md) with its evidence.

## Release Procedure

1. Pushing a `v*` tag runs the release workflow: it builds the Windows, macOS (Apple Silicon and Intel), and Linux files and **starts each one on its OS**. If the Windows or macOS check, or `cargo deny`, fails, no release is created.
2. On success it creates a **draft** release only.
3. A maintainer runs the [manual checklist](specs/004-launch-reliability-refactor/quickstart.md) on clean Windows 10 and 11 VMs and a real Mac, and attaches the result as `launch-verification-manual.md` to the draft.
4. Only then is it published.

### Coding Standards
- Follow Test-Driven Development (write failing tests before production fixes).
- Preserve Zero-Trust path, archive, and HTTPS update invariants (no `http://` or test-only security bypasses).
- Keep user-facing commit summaries in Korean per project governance (`.specify/memory/constitution.md` Principle I), and code/comments/specs in English (Principle II), with Korean default (`*.md`) and English counterpart (`*.en.md`) root documentation.
