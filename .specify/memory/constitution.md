<!--
Sync Impact Report:
- Version change: 1.3.0 -> 1.4.0 (MINOR: eliminated ambiguous loopholes, deduplicated Principles V–VII, and sharpened CWE-22/88/190/409 security & governance rules)
- Modified principles:
  - IV. Zero Hardcoding, Mocking, or Test-Bypass Logic (NON-NEGOTIABLE) (explicitly prohibited test-only production flags and assertion weakening)
  - V. Evidence-Based Research & Verification (NON-NEGOTIABLE) (removed subjective "non-trivial" loophole; specified mandatory evidence scope)
  - VI. Skeptical Verification & Fallibility Awareness (NON-NEGOTIABLE) (focused cleanly on reliability, concurrency/disposal, and falsification without duplicating security rules)
  - VII. Zero Trust Security & Untrusted Input Stance (NON-NEGOTIABLE) (added CWE-22 trailing-separator & reparse-point rules, CWE-190 checked dimension arithmetic, and CWE-88 shell quoting rules)
- Modified sections:
  - Technical Stack & Architectural Constraints (consolidated factual baseline constants without repeating normative prose)
  - Development Workflow & Quality Gates (removed subjective "non-trivial" qualifier; aligned gates 1:1 with Principles I–VII)
  - Governance (closed unilateral-exception loophole on NON-NEGOTIABLE rules; aligned CRITICAL conflict rule with Spec Kit analysis)
- Added sections: None
- Removed sections: None
- Follow-up TODOs: None (remove this HTML comment block before committing if desired)
-->
# ImageViewer Constitution

The key words `MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`,
`RECOMMENDED`, `MAY`, and `OPTIONAL` in this document are to be interpreted as described in
RFC 2119 and RFC 8174 when written in ALL CAPS.

## Core Principles

### I. Korean Communication & Commit Policy

All user-facing questions, clarification prompts, answers, status summaries, review reports,
and Git commit messages MUST be written in Korean. Consistent Korean communication ensures
unambiguous alignment with the maintainer and preserves readability across the repository's
Korean commit history.

- **Compliance Check**: Every user-facing message and proposed or executed Git commit
  message is verified to be written in Korean before delivery.

### II. English-First Reasoning & Code Implementation

Internal reasoning, technical analysis, architectural design thought, and new or modified
code implementation MUST default to English. All new or modified code identifiers
(namespaces, types, methods, properties, parameters, locals), test names, XML docstrings,
and implementation comments MUST be written in English.

- **Legacy & Localization Boundary**: Existing untouched Korean comments in legacy files MAY
  remain until those lines are modified; user-facing UI strings and satellite resources MUST
  preserve the project's `ko;en` localization configuration.
- **Compliance Check**: Diffs in `src/` and `tests/` contain English identifiers, tests,
  docstrings, and comments across all added or modified lines.

### III. Test-Driven Development (NON-NEGOTIABLE)

All new features, bug fixes, and behavioral modifications MUST follow Classical
(Detroit-school) Test-Driven Development (TDD) using `tests/ImageViewer.Tests` and
`tests/ImageViewer.IntegrationTests` (`xUnit`). This principle explicitly supersedes any
downstream Spec Kit template default marking tests as optional.

- **Red-Green-Refactor**: Contributors MUST write failing unit or integration tests first,
  observe the failure for the expected reason, implement the minimal authentic production
  code required to pass, and refactor only while keeping the entire test suite green.
- **State Over Interaction**: Tests MUST assert observable state, outputs, and domain
  invariants against real objects rather than brittle internal call-order expectations.
- **Compliance Check**: Task lists order test tasks prior to implementation tasks, and final
  verification runs `dotnet test ImageViewer.sln` after the last edit with zero skipped or
  weakened assertions.

### IV. Zero Hardcoding, Mocking, or Test-Bypass Logic (NON-NEGOTIABLE)

Hardcoded outputs, mock objects or mocking frameworks substituting for real behavior,
stubbed production paths, and conditional bypass logic written solely to pass tests MUST
NOT be introduced anywhere in the codebase.

- **Prohibited Practices**: Hardcoded test-matching return values in `src/`, interaction-scripted
  mocks (especially mocking third-party libraries we do not own such as `Magick.NET`,
  `SkiaSharp`, or `SharpCompress`), `#if` or environment switches in `src/` that alter
  production semantics only during tests, and `[Fact(Skip = ...)]` or relaxed tolerances
  added to force a passing run.
- **Authentic Test Boundaries**: Tests MUST exercise genuine decoding, archive, caching, and
  state-transition logic using real in-memory or temporary-directory fixtures. Only external
  OS boundaries that cannot execute headlessly (such as modal dialogs or shell registry
  hooks) MAY sit behind narrow, domain-owned interfaces (`IDialogService`, `IImageSource`)
  backed by contract-faithful implementations.
- **Compliance Check**: Diff inspection confirms zero mocking-framework packages, zero
  test-only conditional branches in `src/`, and authentic input-to-output assertions.

### V. Evidence-Based Research & Verification (NON-NEGOTIABLE)

During code review, verification, evaluation, benchmarking, architectural planning, and
root-cause analysis, contributors MUST NOT rely solely on unaided intuition, memory, or
unverified assumptions. Following Evidence-Based Software Engineering (EBSE) discipline,
every technical claim, library usage decision, and evaluation verdict MUST be grounded in
evidence actively retrieved via research tools or direct codebase/runtime inspection.

- **Mandatory Source Hierarchy**: Every architectural choice, external library API usage
  (`Magick.NET`, `SkiaSharp`, `SharpCompress`, `.NET`, `WPF`, Windows Shell), concurrency or
  caching design, security control, and performance claim MUST cite primary evidence from
  official documentation and references, official guides, authoritative community examples,
  peer-reviewed papers, or current industry standards.
- **Traceable Reporting**: Research artifacts (`research.md`), design plans (`plan.md`), and
  review evaluations MUST cite exact sources or measured tool outputs and MUST separate
  directly observed facts from inferences or environment-blocked gaps.
- **Compliance Check**: Every research artifact, plan, and review evaluation lists traceable
  external citations or empirical command outputs backing each technical judgment.

### VI. Skeptical Verification & Fallibility Awareness (NON-NEGOTIABLE)

No implementation, passing test suite, third-party library, specification, or AI-generated
output MAY ever be assumed complete, bug-free, or infallible ("nothing is perfect; always
question"). Grounded in Edsger W. Dijkstra's falsification principle and reliability
engineering ("hope is not a strategy"), every artifact MUST be treated as a fallible
hypothesis subject to adversarial scrutiny and defense-in-depth safeguards.

- **Reliability & Concurrency Scrutiny**: Contributors MUST actively seek counter-examples
  across concurrency cancellation races, out-of-order async completions, `IDisposable`
  lifetime leaks, bounded-cache eviction edges, empty/boundary collections, and extreme
  viewport or DPI scaling states rather than validating happy paths alone.
- **Defense in Depth & Fail-Safe Defaults**: Critical invariants (resource cleanup, version
  or cancellation tokens, settings recovery, bounded queues) MUST enforce fail-safe defaults
  and independent guards instead of assuming upstream callers behave correctly.
- **No False Confidence from Green Runs**: Passing tests, clean builds, or prior reviews MUST
  NOT be cited as proof that defects are absent; every unverified execution path, host OS
  limitation, and residual assumption MUST be surfaced explicitly.
- **Compliance Check**: Specifications, plans, and reviews document explicit failure-mode
  analyses, negative/boundary test coverage, and residual risks or unverified paths rather
  than claiming absolute correctness.

### VII. Zero Trust Security & Untrusted Input Stance (NON-NEGOTIABLE)

Contributors MUST NEVER assume users are rational, benign, or operating within normal usage
patterns; the first principle of security is Zero Trust ("never trust, always verify" and
"assume breach", per NIST SP 800-207 and OWASP Input Validation guidance). Every external
input, user action, file, archive, and OS boundary MUST be treated as potentially malformed,
hostile, or compromised.

- **Never Trust User or File Metadata**: Command-line arguments, drag-and-drop paths,
  filesystem names, ZIP/CBZ archive entries, image payloads, EXIF/metadata tags, Windows
  shell verb arguments, and persisted `settings.json` files MUST be validated as untrusted
  data. File formats MUST be verified via actual binary headers/magic bytes rather than
  extensions alone, strictly rejecting unsupported or scriptable formats (SVG, PDF, PS).
- **Resource, Path, Arithmetic & Shell Boundaries**: Pipelines MUST use overflow-safe
  64-bit/checked dimension arithmetic (mitigating CWE-190) and enforce pre-allocation
  pixel-count and streaming uncompressed-byte caps (mitigating CWE-409 decompression bombs).
  Path normalization MUST enforce trailing `Path.DirectorySeparatorChar` boundary checks on
  canonicalized roots and reject symlinks or NTFS `ReparsePoint` traversal (CWE-22 / Zip
  Slip). Shell verb registration (`--register-shell`) MUST properly quote expansion targets
  against argument injection (CWE-88) under HKCU least privilege, and logs MUST omit full
  host paths (retaining filename-only logging).
- **Fail Closed & Safe Recovery**: Malformed inputs, spoofed headers, corrupted settings, or
  ambiguous trust checks MUST fail closed—rejecting unsafe payloads with safe diagnostics or
  reverting to secure defaults without crashing or executing arbitrary content.
- **Compliance Check**: Every specification, plan, and change touching inputs, files,
  archives, CLI/shell handling, or settings documents its trust boundaries and includes
  negative security tests (spoofed extensions, malformed headers, bomb/oversized inputs,
  traversal/reparse-point paths, quoted shell args, and corrupt configuration recovery).

## Technical Stack & Architectural Constraints

- **Target Platform & Runtime**: Windows 10 / 11 (x64 only), `.NET 10` (`net10.0-windows`),
  WPF (`UseWPF`), `Nullable` enabled, `ImplicitUsings` enabled, `LangVersion` latest.
- **Core Libraries**: `CommunityToolkit.Mvvm` (MVVM), `Magick.NET-Q8-OpenMP-x64` (multi-core
  image decoding and resizing), `SkiaSharp` (GIF/WebP animation streaming), `SharpCompress`
  (ZIP/CBZ archive reading), and `Microsoft.Extensions.DependencyInjection` /
  `Microsoft.Extensions.Logging`.
- **Testing Frameworks**: `xUnit` (`Microsoft.NET.Test.Sdk`, `xunit.runner.visualstudio`)
  across `tests/ImageViewer.Tests` (unit tests) and `tests/ImageViewer.IntegrationTests`
  (integration tests) with `InternalsVisibleTo` granted by `ImageViewer`.
- **Safety & Performance Baseline**: Default safety caps enforce `150 MP`
  (`MaxImageMegapixels`) pre-decode pixel limits and `512 MB` (`MaxZipEntryMegabytes` /
  `DefaultBudgetBytes`) archive/animation memory budgets, bounded LRU caches, header-based
  format rejection (SVG, PDF, PS), and non-destructive in-memory display transforms.
- **Localization**: `SatelliteResourceLanguages` is configured as `ko;en`; user-facing UI
  resources preserve bilingual localization while code and tests remain English-first.

## Development Workflow & Quality Gates

- **Phase 0 Research Gate (`/speckit-plan`)**: Every architectural decision, external library
  API usage, concurrency/caching design, security control, and performance claim MUST cite
  verified official references or empirical measurements (Principle V).
- **TDD & Task Ordering Gate (`/speckit-tasks`, `/speckit-implement`)**: Test tasks MUST
  precede production implementation tasks per feature story, enforcing Red-Green-Refactor
  without skipping or weakening assertions (Principle III).
- **Anti-Bypass Audit Gate (`/speckit-analyze`, Review)**: All diffs MUST be audited to
  verify zero hardcoding, zero mocking of owned or third-party logic, zero test-only
  production branches, and authentic input-to-output checks (Principle IV).
- **Skeptical & Adversarial Review Gate (`/speckit-analyze`, Review)**: Every design, diff,
  and test suite MUST be challenged for unstated assumptions, concurrency/disposal races,
  boundary states, and unverified execution paths (Principle VI).
- **Zero Trust Security Gate (`/speckit-specify`, `/speckit-plan`, Review)**: All input,
  archive, decoder, CLI/shell, and persistence boundaries MUST document threat assumptions,
  enforce magic-byte/arithmetic/resource/path/quoting bounds, fail closed, and include
  negative security tests (Principle VII).
- **Build & Test Verification Gate**: `dotnet build ImageViewer.sln` and
  `dotnet test ImageViewer.sln` MUST pass cleanly after the final edit; if a non-Windows
  host cannot execute `net10.0-windows` targets, that platform limitation MUST be reported
  verbatim as an unverified gap rather than waived or fabricated.
- **Language Gate**: User-facing Q&A and Git commit messages MUST be Korean (Principle I),
  while reasoning, code, tests, docstrings, and implementation comments MUST default to
  English (Principle II).

## Governance

- **Supremacy**: This constitution supersedes all informal habits, template defaults, and
  convenience shortcuts across the `ImageViewer` repository. Any conflict between this
  constitution and a spec, plan, or task list is `CRITICAL` and MUST be resolved by amending
  the downstream artifact rather than diluting a constitutional principle.
- **No Unilateral Waivers**: `NON-NEGOTIABLE` principles (`III`–`VII`) and language mandates
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
  adherence to Principles I–VII and record verification evidence explicitly.

**Version**: 1.4.0 | **Ratified**: 2026-11-15 | **Last Amended**: 2026-11-15
