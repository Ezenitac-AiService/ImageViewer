# ImageViewer Constitution

The key words `MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`,
`RECOMMENDED`, `MAY`, and `OPTIONAL` in this document are to be interpreted as described in
RFC 2119 and RFC 8174 when written in ALL CAPS.

## Core Principles

### I. Korean Communication, Commit & GitHub Documentation Policy

All user-facing questions, clarification prompts, answers, status summaries, review reports,
and Git commit messages MUST be written in Korean. In addition, all public-facing GitHub
repository documents (`README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`,
`THIRD_PARTY_NOTICES.md`, and `NOTICE`) MUST be authored in **Korean as the default landing
page (`*.md`)**, paired with a synchronized **English counterpart (`*.en.md`)** (`README.en.md`,
`CONTRIBUTING.en.md`, `SECURITY.en.md`, `CHANGELOG.en.md`, `THIRD_PARTY_NOTICES.en.md`, and
`NOTICE.en.md`).

- **Bidirectional Language Switching & Issue Templates**: Every root GitHub document (`*.md`)
  and its English counterpart (`*.en.md`) MUST include a top-level bidirectional language
  switcher link (`**언어 / Language**: **[한국어](...)** | [English](*.en.md)`), and
  `.github/ISSUE_TEMPLATE/` forms MUST provide bilingual (`한국어 / English`) labels and prompts.
- **Exclusion of Internal Agent & Spec Directories**: Internal agent, governance, and
  specification directories (`.agents/`, `.specify/`, and `specs/`) are explicitly excluded
  from Korean document pairing and remain governed by Principle II (English).
- **Compliance Check**: Every user-facing message and Git commit message is written in Korean,
  and any update to a root GitHub document updates both the Korean default (`*.md`) and English
  counterpart (`*.en.md`) in lockstep with working cross-links.

### II. English-First Reasoning, Specs & Code Implementation

Internal reasoning, technical analysis, architectural design thought, Spec-Kit artifacts under
`.agents/`, `.specify/`, and `specs/`, and all new or modified code implementation MUST default
to English. All Rust crate names, modules, structs, enums, traits, functions, parameters,
locals, test names, rustdoc comments (`///`), and inline implementation comments MUST be
written in English.

- **Localization & Documentation Boundary**: Root GitHub-facing documents follow Principle I's
  Korean-default (`*.md`) + English-counterpart (`*.en.md`) policy; user-facing UI strings,
  localized error messages, and resource tables MUST preserve complete Korean (`ko`) and
  English (`en`) localization.
- **Compliance Check**: Diffs across workspace crates (`crates/`, `packaging/`) and Spec-Kit
  directories (`.agents/`, `.specify/`, `specs/`) contain English identifiers, specs, tests,
  rustdoc, and comments across all added or modified lines.

### III. Test-Driven Development (NON-NEGOTIABLE)

All new features, bug fixes, and architectural modules MUST follow Classical (Detroit-school)
Test-Driven Development (TDD) using Cargo workspace unit and integration test suites
(`cargo test`). This principle explicitly supersedes any downstream Spec Kit template default
marking tests as optional.

- **Red-Green-Refactor**: Contributors MUST write failing unit or integration tests first,
  observe the failure for the expected reason, implement the minimal authentic production
  code required to pass, and refactor only while keeping the entire test suite green.
- **State Over Interaction**: Tests MUST assert observable state, outputs, decoded pixels,
  rendered geometry invariants, and security rejections against real fixtures rather than
  brittle internal call-order expectations.
- **Compliance Check**: Task lists order test tasks prior to implementation tasks, and final
  verification runs `cargo test --workspace` after the last edit with zero ignored (`#[ignore]`)
  or weakened assertions.

### IV. Zero Hardcoding, Mocking, or Test-Bypass Logic (NON-NEGOTIABLE)

Hardcoded outputs, mock objects or mocking frameworks substituting for real domain behavior,
stubbed production paths, and conditional compilation or environment bypass logic written
solely to pass tests MUST NOT be introduced anywhere in production crates.

- **Prohibited Practices**: Hardcoded test-matching return values in production crates,
  interaction-scripted mocks substituting for owned decoders/archives/caches/state machines,
  `#[cfg(test)]`, `#[cfg(debug_assertions)]`, or environment variables inside production code
  that weaken security checks (such as bypassing HTTPS/checksum verification or limits), and
  `#[ignore]` or loosened tolerances added to force a passing test run.
- **Safe Rust & Authentic OS Boundaries**: Core security validation, archive parsing, path
  canonicalization, settings normalization, and state orchestration modules MUST avoid `unsafe`
  code wherever practical. External OS boundaries that cannot execute identically across all
  headless test hosts (modal file dialogs, host OS platform image codecs, OS shell/default-app/
  bundle registration, timers/clock, UI dispatcher) MAY sit behind narrow, domain-owned traits
  backed by contract-faithful implementations and audited `// SAFETY:` invariants on FFI calls.
- **Compliance Check**: Diff inspection confirms zero test-only production behavior flags, zero
  `#[ignore]` bypasses, minimal isolated `unsafe` blocks with documented invariants, and
  authentic input-to-output assertions.

### V. Evidence-Based Research & Verification (NON-NEGOTIABLE)

During code review, verification, evaluation, benchmarking, architectural planning, and
root-cause analysis, contributors MUST NOT rely solely on unaided intuition, memory, or
unverified assumptions. Following Evidence-Based Software Engineering (EBSE) discipline,
every technical claim, crate selection, performance benchmark claim, and evaluation verdict
MUST be grounded in evidence actively retrieved via research tools or direct codebase/runtime
inspection.

- **Mandatory Source Hierarchy**: Every architectural choice, Rust crate selection (GUI/GPU
  rendering, SIMD scaling, core image/archive decoders, EXIF, async runtimes, OS bindings),
  platform codec integration (`ImageIO` on macOS and iOS, and optional fallback codecs on
  build-only Linux and Android targets), security control, and performance claim MUST cite
  primary evidence from official crate/OS documentation, authoritative benchmarks,
  peer-reviewed guidance, or industry standards.
- **Traceable Reporting**: Research artifacts (`research.md`), design plans (`plan.md`), and
  review evaluations MUST cite exact sources or measured tool outputs and MUST separate
  directly observed facts on the current host OS from inferences or unverified target-OS gaps.
- **Compliance Check**: Every research artifact, plan, and review evaluation lists traceable
  external citations or empirical command outputs backing each technical judgment.

### VI. Skeptical Verification & Fallibility Awareness (NON-NEGOTIABLE)

No implementation, passing test suite, third-party crate, OS codec, specification, or
AI-generated output MAY ever be assumed complete, bug-free, or infallible ("nothing is
perfect; always question"). Grounded in Edsger W. Dijkstra's falsification principle and
reliability engineering ("hope is not a strategy"), every artifact MUST be treated as a
fallible hypothesis subject to adversarial scrutiny and defense-in-depth safeguards.

- **Reliability & Concurrency Scrutiny**: Contributors MUST actively seek counter-examples
  across async task cancellation races, out-of-order decode completions, deterministic RAII
  `Drop` resource/GPU-texture cleanup, bounded-channel/LRU eviction edges, high-DPI scaling,
  macOS/iOS app and window lifecycle, pointer/touch, and drag-and-drop differences, and
  missing OS platform codecs rather than validating happy paths alone.
- **Defense in Depth & Fail-Safe Defaults**: Critical invariants (cancellation versioning,
  pixel/memory budgets, settings recovery, bounded channels, graceful shutdown flushes) MUST
  enforce fail-safe defaults and independent guards instead of assuming upstream callers or
  external libraries behave safely.
- **No False Confidence from Green Runs**: Passing tests on a single developer host OS MUST
  NOT be cited as proof that all top-priority targets (`macOS`, `iOS`) are verified, and
  passing library tests MUST NOT be cited as proof that a packaged artifact launches;
  build-only lower-priority targets (`Linux`, `Android`), unverified hardware rendering paths,
  and residual risks MUST be disclosed explicitly as unverified.
- **Compliance Check**: Specifications, plans, and reviews document explicit failure-mode
  analyses, negative/boundary test coverage, and residual cross-OS risks.

### VII. Zero Trust Security & Untrusted Input Stance (NON-NEGOTIABLE)

Contributors MUST NEVER assume users are rational, benign, or operating within normal usage
patterns; the first principle of security is Zero Trust ("never trust, always verify" and
"assume breach", per NIST SP 800-207 and OWASP Input Validation guidance). Every external
input, user action, file, archive, remote release feed, and OS boundary MUST be treated as
potentially malformed, hostile, or compromised.

- **Never Trust User or File Metadata**: CLI arguments, drag-and-drop URIs/paths, directory
  entries, ZIP/CBZ archive headers, image payloads, EXIF/ICC tags, OS file-association/shell
  commands, remote update manifests/packages, and persisted `settings.json` files MUST be
  validated as untrusted data. Formats MUST be verified via leading binary signatures before
  decoding, strictly rejecting disguised scriptable/document payloads (SVG, MVG, PDF, PS, HTML).
- **Resource, Path, Arithmetic, Transport & Shell Boundaries**: Pipelines MUST use
  overflow-safe checked dimension/stride arithmetic (CWE-190) and enforce pre-allocation
  pixel-count and streaming uncompressed/HTTP byte caps (CWE-400 / CWE-409 decompression bombs).
  Path canonicalization MUST enforce root-boundary delimiter checks and reject symlinks or
  filesystem link/reparse escapes in archives and app-managed config/log/temp dirs while safely
  skipping symlinks during folder enumeration (CWE-22 / Zip Slip). OS command/shell/bundle
  integrations MUST reject injection characters and perform exact command matching (CWE-88),
  remote updates MUST enforce unconditional `https://` + SHA-256 verification (CWE-494), and
  logs/UI error prompts MUST redact full host directory paths and strip CR/LF control chars
  against log injection (CWE-532 / CWE-117).
- **Fail Closed & Safe Recovery**: Malformed inputs, spoofed headers, missing optional OS
  codecs, corrupted/oversized settings (`> 64 KB`), and ambiguous trust checks MUST fail
  closed with safe localized diagnostics or secure finite defaults without crashing.
- **Compliance Check**: Every specification, plan, and change touching inputs, files,
  archives, decoders, updates, CLI/OS hooks, or persistence documents trust boundaries and
  includes automated negative security tests.

### VIII. License Compliance, Attribution & Independent Stewardship (NON-NEGOTIABLE)

`Ezenitac-AiService/ImageViewer` operates as an independent, detached cross-platform fork of
`edith2580/ImageViewer` (`v1.0.6`). Contributors MUST uphold legal copyleft compliance, full
third-party attribution, and courteous upstream boundaries at all times.

- **GPLv3 Continuity & §5 Notices**: The repository root `LICENSE` (`GNU GPLv3`) and all workspace
  package manifests (`Cargo.toml` `license = "GPL-3.0-only"`) MUST remain consistent. Original
  `edith2580` copyright notices MUST be preserved intact alongside `Ezenitac-AiService` notices
  and prominent GPLv3 §5(a)/(d) modification and date notices (`NOTICE` / `NOTICE.en.md`,
  `CHANGELOG.md` / `CHANGELOG.en.md`, `README.md` / `README.en.md`, and the in-app `About` dialog).
- **Third-Party Dependency & Compatibility Gate**: All bundled and transitive dependencies MUST be
  compatible with `GPL-3.0-only` (verified via `deny.toml` / `cargo-deny`), with framework and
  library notices published in both `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_NOTICES.en.md`.
- **Detached Identity & Courteous Boundaries**: Runtime update feeds, releases, and OS bundle
  identifiers (`com.ezenitac.imageviewer`) MUST target exclusively `Ezenitac-AiService/ImageViewer`.
  Direct `git merge upstream` synchronization is prohibited; desired upstream ideas MUST be
  re-specified and re-implemented independently, and `README.md` / `.github/ISSUE_TEMPLATE/` MUST
  courteously route cross-platform fork issues away from the upstream maintainer.
- **Compliance Check**: Audits confirm zero non-GPLv3 manifest declarations, zero unlicensed or
  GPLv3-incompatible dependencies, and zero stray upstream or placeholder update endpoints.

## Technical Stack & Architectural Constraints

- **Target Platforms (Tiered)**: Desktop and mobile targets are classified strictly into three
  governance categories:
  - **Top Priority (Supported — Build & Verification)**: **`macOS` (Apple Silicon `aarch64` &
    Intel `x86_64`)** and **`iOS` (`aarch64`)**. Every release artifact for a top-priority target
    MUST be built AND pass build-result / launch verification on that target (the packaged
    artifact starts, displays its UI window, renders a reference image, and exits cleanly)
    before publication; any failure MUST block publication. Build-result verification is
    performed **exclusively** for this tier.
  - **Lower Priority (Build-Only — No Verification)**: **`Linux` (`x86_64`, X11 & Wayland)**
    and **`Android` (`aarch64` / configured Android ABIs)**. Lower-priority targets provide
    **build artifacts only**; CI and release pipelines MUST NOT perform build-result or launch
    verification on them, MUST NOT block publication on their runtime behavior, and MUST label
    them as build-only and unverified in release notes and documentation (Principle VI).
  - **Excluded**: **`Windows`** is excluded from both supported and build-only targets. Active
    development, packaging, and verification pipelines MUST NOT require Windows support
    (historical Windows C#/.NET `v1.0.6` source remains preserved solely under `legacy-v1.0.6`
    and `specs/001-*`).
  - **Promotion**: A lower-priority target may be promoted to top priority only after passing
    the full build-result and launch verification required of top-priority targets, recorded as
    evidence, and only through a formal amendment of this section.
  - **Rationale**: Concentrating build-result verification exclusively on top-priority Apple
    platforms (`macOS`, `iOS`) keeps release-blocking quality gates enforceable and focused,
    while continuing to supply `Linux` and `Android` as build-only artifacts with honest
    unverified disclosure (Principle VI) and retiring `Windows` from active maintenance.
- **Language & Workspace Architecture**: Stable **Rust** Cargo workspace separating
  UI-independent core logic (domain models, path/format/stream security guards, layout/zoom
  math, session/prefetch/slideshow orchestration, decoder pipeline, settings/logging) from
  cross-platform GPU-accelerated UI presentation (`wgpu` / native-feeling UI framework) and
  narrow OS adapter modules (Apple AppKit/UIKit/Foundation for `macOS`/`iOS`, and build-only
  adapters for `Linux`/`Android`).
- **Media Decoding Strategy (Target, Delivered by Capability Tier)**: The **end-state target** is
  built-in memory-safe pure-Rust/SIMD core decoders for standard raster, animated, layered,
  multi-page, and HDR formats (JPEG with fast viewport-scaled decoding, PNG/APNG, GIF, BMP, TIFF,
  WebP, ICO/CUR, JXL, TGA, PSD, DDS, PCX, QOI, PNM, HDR, EXR) plus read-only `.zip`/`.cbz`
  streaming, and host OS platform imaging codecs (`ImageIO` on `macOS` and `iOS`, with graceful
  fallback or unsupported-format messages on build-only `Linux` and `Android` targets) for
  camera RAW (8 extensions), `.heic`/`.heif`, `.avif`, and `.jp2`. Delivery is governed by
  capability tiers:
  - **Tiers**: Every user-facing capability, including each format family and the OS-codec
    bridge, is classified **Core** (verified on all top-priority targets: `macOS`, `iOS`),
    **Extended** (verified individually), or **Unavailable**, in a published bilingual capability
    register.
  - **Shipped artifact**: A release MUST ship only Core and verified Extended capabilities.
    Unavailable decoders MUST remain in the workspace behind default-off build features, MUST
    compile and pass the full test suite in CI with all features enabled, and MUST NOT be
    reachable or advertised in the shipped product.
  - **Promotion to Extended**: A capability is promoted only with (a) automated tests on real
    sample fixtures, (b) automated hostile/malformed-input tests meeting Principle VII, (c) a
    passing launch-verification record on every top-priority target (`macOS`, `iOS`), and (d)
    user documentation in Korean and English. Demotion is immediate when any of that evidence is
    invalidated.
  - **Principle IV boundary**: Capability-tier build features are product-scope controls, not
    test bypasses. They MUST NOT weaken, skip, or alter any security check in any configuration,
    and the shipped (default) configuration MUST itself be fully tested.
  - **Rationale**: `v1.1.0-alpha.1` advertised the full format set with no format verified end to
    end on a packaged artifact (`specs/004-launch-reliability-refactor/feasibility-review.md`).
    Tiers keep the target while ensuring that everything shipped has evidence behind it.
- **Safety Caps (Mandatory) & Performance Parity Baseline (Target, Gated by Measurement)**:
  - **Safety caps, mandatory now**: Every release MUST enforce the `150 MP` default pre-decode
    pixel cap (`10-1000 MP` clamp), `512 MB` stream/animation cap (`128 MB` forward ring buffer),
    `<= 8` decoded LRU cache, `300`-thumbnail cap (when thumbnails ship), `64 KB` settings cap,
    and `ko`/`en` bilingual localization. These caps admit no tier or deferral.
  - **Performance targets**: The measured `v1.0.6` reference figures remain the target
    (`<= 36 ms` avg single-frame decode+display windowed, `<= 49 ms` avg fullscreen `2560x1440`,
    `<= 22 ms` rapid navigation, `<= 16 ms` slideshow transition, `<= 470 MB` RAM and `<= 5%` CPU
    on high-frame `1080x1920` animated WebP).
  - **Not release-blocking until measurable**: Until an automated end-to-end measurement harness
    runs these scenarios against a packaged artifact on a top-priority target, the figures MUST
    NOT block a release. They also MUST NOT be claimed as met in documentation or release notes;
    claims require harness output (Principle V).
  - **Becomes a regression gate**: Once the harness exists and has recorded a baseline on a
    top-priority target, a release MUST NOT regress any figure beyond that recorded baseline (or
    beyond the `v1.0.6` target, whichever is stricter) without a recorded, reviewed justification.
  - **Interim responsiveness floor**: Until then, releases MUST meet the responsiveness criteria
    of the active feature specification (for `specs/004`: on-screen controls respond within
    `100 ms`; first image of a 10,000-file folder within `1 s`).
- **Legacy Reference Boundary**: Prior C#/.NET 10 WPF `v1.0.6` source is preserved under the
  immutable `legacy-v1.0.6` archive tag and `specs/001-*` (archived baseline) once cut over to
  the pure-Rust workspace on `main`.

## Development Workflow & Quality Gates

- **Phase 0 Research Gate (`/speckit-plan`)**: Every architectural decision, Rust crate,
  SIMD/GPU rendering path, OS codec adapter, security control, and performance claim MUST cite
  verified official references or empirical measurements (Principle V).
- **TDD & Task Ordering Gate (`/speckit-tasks`, `/speckit-implement`)**: Test tasks MUST
  precede production implementation tasks per feature story, enforcing Red-Green-Refactor
  without skipping (`#[ignore]`) or weakening assertions (Principle III).
- **Anti-Bypass & `unsafe` Audit Gate (`/speckit-analyze`, Review)**: All diffs MUST confirm
  zero hardcoding, zero mocking of owned logic, zero test-only production flags, minimal
  audited `unsafe` code with `// SAFETY:` comments, and authentic assertions (Principle IV).
- **Skeptical & Adversarial Review Gate (`/speckit-analyze`, Review)**: Every design, diff,
  and test suite MUST be challenged for async cancellation races, RAII `Drop`/GPU leaks,
  boundary states, and unverified OS platform paths (Principle VI).
- **Zero Trust Security Gate (`/speckit-specify`, `/speckit-plan`, Review)**: All input,
  archive, decoder, remote update, CLI/OS hook, and persistence boundaries MUST document threat
  assumptions, enforce magic-byte/arithmetic/resource/path/HTTPS bounds, fail closed, and
  include negative security tests (Principle VII).
- **Cross-Platform Build, Lint & Test Verification Gate**: `cargo check --workspace`,
  `cargo clippy --workspace --all-targets`, and `cargo test --workspace` MUST pass cleanly on
  the active host after the final edit; any top-priority target (`macOS`, `iOS`) not executed
  natively on the local machine MUST be disclosed explicitly until verified on that OS or CI
  runner. Lower-priority targets (`Linux`, `Android`) require build checks only with zero runtime
  verification; `Windows` is excluded from build and verification gates.
- **Launch Verification Gate (Release)**: Before any release is published, build-result and
  launch verification MUST execute **only** for top-priority targets (`macOS`, `iOS`) as
  defined under Target Platforms; any failure MUST block publication. Lower-priority targets
  (`Linux`, `Android`) deliver build artifacts only without build-result verification, MUST NOT
  run launch verification checks, and MUST be disclosed as build-only/unverified in release
  notes. `Windows` is excluded from release builds and verification. Release notes MUST list
  verified top-priority targets, any unverified top-priority slice, and build-only lower-priority
  targets.
- **Capability Tier Gate (`/speckit-plan`, `/speckit-analyze`, Release)**: Every capability that is
  reachable in the shipped product or claimed in documentation MUST appear in the capability
  register as Core or verified Extended, with links to its evidence. CI MUST build and test both
  the shipped (default) configuration and the all-features configuration.
- **Performance Regression Gate (activates with the measurement harness)**: Once an end-to-end
  performance harness exists, release CI MUST run it on at least one top-priority target and
  fail on regressions as defined under Safety Caps & Performance Parity Baseline. Until then,
  plans and release notes MUST state that performance parity is unmeasured.
- **Language & GitHub Documentation Gate**: User-facing Q&A and Git commit messages MUST be
  Korean, and root GitHub documents (`README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`,
  `THIRD_PARTY_NOTICES.md`, `NOTICE`) MUST maintain Korean default (`*.md`) and synchronized
  English (`*.en.md`) pages with bidirectional links (Principle I), while `.agents/`, `.specify/`,
  `specs/`, and Rust code/tests/rustdoc/comments MUST default to English (Principle II).
- **License & Attribution Gate**: `cargo deny check licenses` (`deny.toml`) and manifest/URL audits
  MUST confirm `GPL-3.0-only` compliance, `THIRD_PARTY_NOTICES.md` / `THIRD_PARTY_NOTICES.en.md`
  currency, and zero upstream update-feed coupling (Principle VIII).

## Governance

- **Supremacy**: This constitution supersedes all informal habits, template defaults, and
  convenience shortcuts across the `ImageViewer` repository. Any conflict between this
  constitution and a spec, plan, or task list is `CRITICAL` and MUST be resolved by amending
  the downstream artifact rather than diluting a constitutional principle.
- **No Unilateral Waivers**: `NON-NEGOTIABLE` principles (`III`–`VIII`) and language mandates
  (`I`–`II`) admit no unilateral runtime exceptions; relaxing or altering any rule requires a
  formal constitution amendment.
- **Amendment Procedure**: Amendments require an explicit proposal stating the rationale,
  updating `.specify/memory/constitution.md` with a refreshed `Last Amended` ISO date, and
  recording version and section changes in a Sync Impact Report.
- **Versioning Policy**: Governed by Semantic Versioning (`MAJOR.MINOR.PATCH`):
  - `MAJOR`: Backward-incompatible governance changes, principle removals, or redefinitions.
  - `MINOR`: New principle or section additions, or materially expanded mandatory rules.
  - `PATCH`: Clarifications, wording improvements, typo fixes, or non-semantic refinements.
- **Compliance Review**: Every Spec Kit command (`/speckit-specify`, `/speckit-plan`,
  `/speckit-tasks`, `/speckit-analyze`, `/speckit-implement`) and code review MUST verify
  adherence to Principles I–VIII and record verification evidence explicitly.

**Version**: 3.0.0 | **Ratified**: 2026-11-15 | **Last Amended**: 2026-11-24
