# Feature Specification: Launch Reliability & Evidence-Driven Refactoring

**Feature Branch**: `004-launch-reliability-refactor`

**Created**: 2026-11-16

**Status**: Implemented on macOS; Windows, Intel macOS, and clean-machine verification pending (2026-11-17)

**Input**: User description: "윈도우, 맥 둘다 실행되지 않아, 공격적인 비판론자의 관점에서 다중 페르소나 심층분석 기법을 사용해서 프로젝트의 기능, 요구사항, 사용한 프레임워크, 라이브러리에 대한, 타당성 검토를 진행하고 리펙토링 스펙을 작성해줘"

**Supporting analysis**: [feasibility-review.md](./feasibility-review.md) — the adversarial multi-persona review that this specification answers. Finding IDs such as `A4` or `B1` below refer to that document.

## Background

The published `v1.1.0-alpha.1` pre-release does not run for its users on Windows or macOS. The review reproduced the macOS half of the problem and established five root causes:

1. No step in development or release ever started the packaged application.
2. The distributed files are not valid deliverables. The macOS package fails the operating system's own integrity check and supports one processor family only. On Windows the user reports that the main window appears but does not respond, a console window opens alongside it, and closing the console closes the application.
3. When the application cannot start, it exits without telling anyone why.
4. Where the application does start, controls do not respond reliably to the mouse.
5. The previous scope asked for everything at once and omitted the requirement that the product start.

This feature restores a trustworthy baseline: the download starts, failures explain themselves, the core viewing experience works, and nothing is published or advertised without evidence that it works.

## Clarifications

### Session 2026-11-16

- Q: Will paid code-signing credentials (Apple Developer ID, Windows code-signing certificate) be available for this release? → A: No. Ship with a valid free (ad-hoc) signature plus first-launch instructions, and keep the release process ready to switch to publisher-verified files when credentials are obtained.
- Q: How far should the must-work Core capability set extend for the next release? → A: The Core set as drafted, plus read-only ZIP/CBZ archive viewing. Everything else stays withdrawn until re-admitted.
- Q: What status should Linux have in this release? → A: Experimental. It keeps building and passing automated checks and a release file is still provided, labelled unverified; a Linux failure does not block publication. The constitution's Target Platforms section is to be amended to match before planning.
- Q: In what environments should pre-release launch verification run? → A: Mixed. An automated launch check runs on hosted build machines for every change and every release; immediately before each release, a maintainer runs a checklist manually on clean Windows 10 and Windows 11 virtual machines and a real Mac. Intel Mac is verified when one is available and otherwise disclosed as unverified.
- Q: What actually happened when the Windows release file was started? → A: (User report) The main window appears but does not respond; a console window opens alongside it; closing the console also closes the application. The application therefore starts on Windows — the failure is an unresponsive main window plus an attached console, not a trust block or missing system component.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The Download Starts on a Clean Computer (Priority: P1)

As a person who downloads ImageViewer from the release page onto an ordinary Windows PC or Mac, I want the application to open when I follow the published instructions, so that I can use it at all.

**Why this priority**: Nothing else has value until this works. It is the reported defect.

**Independent Test**: On a freshly installed machine of each supported kind, download the published file, follow only the published instructions, and confirm the viewer window appears.

**Acceptance Scenarios**:

1. **Given** a clean Windows 10 or Windows 11 machine with nothing extra installed, **When** the user downloads the Windows release file and starts it following the published instructions, **Then** the viewer window appears and no message reports a missing system component.
2. **Given** a clean Mac with an Apple processor, **When** the user downloads the macOS release file and opens it following the published instructions, **Then** the viewer window appears and the operating system never describes the application as damaged or corrupt.
3. **Given** a clean Mac with an Intel processor, **When** the user performs the same steps, **Then** the result is the same as on an Apple-processor Mac.
4. **Given** the operating system shows a trust confirmation for software from an unrecognised publisher, **When** the user reads the published instructions, **Then** the instructions show that exact confirmation and the steps to proceed, in Korean and in English.
5. **Given** the Windows application has started, **When** the user looks at the desktop and taskbar, **Then** exactly one application window is present, with the product's name and icon, and no auxiliary text console.

---

### User Story 2 - A Failed Start Explains Itself (Priority: P1)

As a user whose computer cannot run the application for some reason, I want to be told what went wrong and where to find details, so that I can fix it or report it instead of staring at nothing.

**Why this priority**: The Windows failure cannot even be diagnosed today because the application gives no signal. This story is what makes every other defect findable.

**Independent Test**: Deliberately provoke each known startup failure condition and confirm that a readable explanation and a diagnostic record are produced every time.

**Acceptance Scenarios**:

1. **Given** a computer where the application's preferred way of drawing to the screen is unavailable, **When** the user starts the application, **Then** it still shows images using an alternative method, or, if no method works, shows a message naming the cause category and the location of the diagnostic record.
2. **Given** any failure that prevents the main window from becoming usable, **When** the application stops, **Then** the user has seen a message in their language and the application reports failure to the operating system.
3. **Given** any launch, successful or not, **When** the user opens the diagnostic record, **Then** it states the application version, the operating system and processor family, which display method was chosen, and any fatal error, without revealing full personal folder paths.
4. **Given** a damaged or unwritable settings location, **When** the user starts the application, **Then** it starts with safe defaults and tells the user that settings could not be loaded or saved.

---

### User Story 3 - Core Viewing Works with Mouse or Keyboard (Priority: P1)

As a user viewing photos, I want every visible control to respond when I use it, with either the mouse alone or the keyboard alone, so that the viewer is usable in the way any desktop application is.

**Why this priority**: This is the defect the user actually reported on Windows: the main window appears but does not respond. The review independently found that on-screen controls ignore the mouse until an unrelated event occurs (`A4`–`A7`). Starting is not enough.

**Independent Test**: Complete a scripted viewing session twice on each operating system, once using only the mouse and once using only the keyboard.

**Acceptance Scenarios**:

1. **Given** the viewer is open, **When** the user points at and clicks any on-screen control, **Then** the control shows feedback and performs its action promptly, without needing any other input.
2. **Given** the viewer is open, **When** the user opens an image, a folder, or a ZIP/CBZ archive by the file chooser, by dragging a file onto the window, by passing a path at launch, or by choosing "open with" in the operating system's file manager, **Then** the image is displayed.
3. **Given** a folder of images is open, **When** the user steps forward and backward, zooms, pans, switches fit modes, rotates, flips, and toggles fullscreen, **Then** each action takes effect and the picture stays correct.
4. **Given** a settings or information window is open, **When** the user changes values and confirms or cancels, **Then** the window responds to each interaction and closes as requested.
5. **Given** a secondary window is open, **When** the user presses the cancel key, **Then** only that window closes and the viewer remains.
6. **Given** the user has navigated away from the image first opened, **When** files in the folder are added, changed, or removed, **Then** the viewer stays on the image currently shown, unless that image itself was removed.
7. **Given** the viewer window, **When** the user moves, resizes, minimises, maximises, or closes it, **Then** it behaves as other applications on that operating system do.
8. **Given** a computer set to Korean, **When** the application starts for the first time, **Then** the interface is in Korean, and the user can switch language from within the application.
9. **Given** the viewer window has just appeared on Windows or macOS, **When** the user waits 10 seconds without any input and then clicks a control, **Then** the operating system has never marked the window as not responding and the control performs its action.

---

### User Story 4 - Nothing Is Published Unless It Has Been Started (Priority: P2)

As a maintainer, I want the release process to refuse to publish any file that has not been automatically started and exercised on its target operating system, so that a broken download cannot reach users again.

**Why this priority**: This is the control whose absence caused the incident. Without it the first three stories will regress.

**Independent Test**: Introduce a deliberate packaging fault and confirm that the release process stops before publication and reports which target failed.

**Acceptance Scenarios**:

1. **Given** release files have been produced, **When** the release process runs, **Then** each file is started on its target operating system, shows its window, displays a reference image, and exits cleanly before anything is published.
2. **Given** any target fails that check, **When** the release process continues, **Then** publication is blocked and the failing target and reason are reported.
3. **Given** a published release, **When** a user reads its notes, **Then** they list which operating systems and processor families were verified, which are unverified or experimental, and the known limitations.
4. **Given** a proposed change, **When** automated checks run before it is merged, **Then** they include starting the application and performing the core interactions, not only checks of internal logic.

---

### User Story 5 - The Product Only Promises What Works (Priority: P3)

As a user, I want every button, menu entry, file association, and claim in the documentation to correspond to something that works, so that I am not misled by features that exist in name only.

**Why this priority**: The previous scope produced many partial features (`F3`, `E5`). Hiding them costs little and removes a large class of visible defects, but it depends on the core being sound first.

**Independent Test**: Compare every user-visible feature and documented claim against the list of verified capabilities and confirm there are no extras.

**Acceptance Scenarios**:

1. **Given** the list of capabilities classified as Core, Extended, or Unavailable, **When** the user explores the application, **Then** only Core capabilities and verified Extended capabilities are reachable.
2. **Given** a capability classified as Unavailable, **When** the user reads the documentation, **Then** it is described as not yet available or not mentioned as a feature.
3. **Given** the user opens a file of a kind that is not currently supported, **When** the application declines it, **Then** it says the format is not supported in this version instead of failing or showing a generic error.
4. **Given** update checking is not a verified capability, **When** the application runs, **Then** it makes no network connections.

---

### User Story 6 - A Foundation That Can Be Audited (Priority: P3)

As a maintainer or security reviewer, I want the product's safety claims to match its tested behaviour and every bundled third-party component to have a stated reason to exist, so that the project can be reviewed and extended with confidence.

**Why this priority**: The review found documented protections that the product does not enforce (`D1`, `D2`) and a component inventory out of proportion to verified capability (Persona G). These are real risks but do not block users today.

**Independent Test**: Trace each documented safety claim to a passing negative test, and each bundled component to a verified capability.

**Acceptance Scenarios**:

1. **Given** a file whose content does not positively identify it as a supported image kind, **When** the user tries to open it, **Then** it is refused before any attempt to decode it, whatever its file name says.
2. **Given** a file larger than the configured size limit, **When** the user tries to open it, **Then** it is refused before its content is loaded into memory.
3. **Given** the inventory of bundled third-party components, **When** a reviewer inspects it, **Then** each component is linked to at least one verified capability, and components serving only Unavailable capabilities are absent from the shipped product.
4. **Given** a bundled component with a publicly known vulnerability, **When** the release process runs, **Then** publication is blocked until the issue is resolved or explicitly accepted with a recorded reason.
5. **Given** an interaction rule such as "what the cancel key does" or "what a click on empty canvas does", **When** a maintainer runs the automated checks, **Then** that rule is verified without needing a screen.

---

### Edge Cases

- The download is opened directly from inside its archive or disk image without being copied anywhere first.
- The application is started from a location the user cannot write to.
- Two copies of the application are started at once.
- The computer has no graphics acceleration, is a virtual machine, or is being used through a remote-desktop session.
- The display uses a scaling factor other than 100 %, or the window is moved between displays with different scaling.
- The settings file left by an earlier version is present, damaged, or larger than permitted.
- The user opens a folder containing tens of thousands of files, or a folder on a slow network location.
- The file named at launch no longer exists, or more than one file is named.
- The operating system hands the application a file to open while it is already running.
- An image is removed from disk while it is being displayed.
- The user's language is neither Korean nor English.
- On Windows, the user starts the application from a command prompt and then closes that prompt; the viewer must keep running.
- The operating system blocks the application outright under a strict security policy that offers no way to proceed.

## Requirements *(mandatory)*

### Functional Requirements

**Launch and distribution**

- **FR-001**: Each published release file MUST start on a clean installation of its target operating system without the user installing any additional system component.
- **FR-002**: The macOS release MUST pass the operating system's structural signature validation of the application package: a strict, deep verification of the package's code signature reports no errors, so the system never describes it as damaged. Assessment as coming from an identified, notarized publisher is not required until signing credentials exist (FR-005). The release MUST run on both Apple-processor and Intel-processor Macs.
- **FR-003**: The Windows release MUST present a single application window carrying the product name, icon, and version, with no auxiliary console window. The application's lifetime MUST be governed only by its own windows: closing any other window, including a command prompt it was started from, MUST NOT end it.
- **FR-004**: The project MUST publish first-launch instructions, in Korean and English, that show the exact trust confirmation each operating system presents and how to proceed.
- **FR-005**: This release ships without paid publisher credentials. Its files MUST nevertheless carry a structurally valid signature where the operating system expects one, so that the only obstacle a user meets is the documented one-time trust confirmation. The release process MUST be able to produce publisher-verified files as soon as signing credentials are supplied, without changes to the product itself.
- **FR-006**: Version and product identity shown to the operating system MUST come from a single source so that they cannot disagree between the application, its package, and its release.

**Failure visibility**

- **FR-007**: The application MUST NOT end a launch attempt silently. Any failure before the main window is usable MUST produce a message in the user's language stating the cause category and where the diagnostic record is kept, and MUST report failure to the operating system.
- **FR-008**: Every launch MUST write a diagnostic record containing application version, operating system, processor family, the display method chosen, and any fatal error. Existing rules on removing personal paths and control characters from records continue to apply.
- **FR-009**: When the preferred display method is unavailable, the application MUST fall back to an alternative that can still show images, and only fail under FR-007 when no method works.
- **FR-010**: Errors encountered during normal use MUST be either shown to the user or written to the diagnostic record. Discarding an error without trace is not permitted.

**Core viewing**

- **FR-011**: Every on-screen control, in every window, MUST give visible feedback and perform its action within 100 ms of a pointer or keyboard interaction, without requiring any other input to occur.
- **FR-012**: Users MUST be able to open an image or folder by file chooser, by drag and drop, by a path supplied at launch, and by the operating system's "open with" action, on both Windows and macOS.
- **FR-013**: Users MUST be able to step between images, zoom, pan, switch fit modes, rotate, flip, enter and leave fullscreen, and close the viewer, by mouse alone and by keyboard alone.
- **FR-014**: The viewer window MUST use the operating system's standard means of moving, resizing, minimising, maximising, and closing.
- **FR-015**: The cancel key MUST close the topmost secondary window if one is open, and only otherwise act on the main viewer.
- **FR-016**: When the contents of the open folder change, the viewer MUST remain on the image currently displayed unless that image was removed.
- **FR-017**: Opening a folder MUST NOT make the interface unresponsive, regardless of the number of files it contains.
- **FR-018**: On first launch the interface language MUST follow the operating system (Korean for Korean systems, English otherwise). Users MUST be able to change it in the application, and every user-visible text in Core capabilities MUST exist in both languages, including status and error messages that originate from internal failures (for example decoding or archive errors).
- **FR-019**: Settings saved by earlier versions MUST continue to load, and unusable settings MUST be replaced by safe defaults with notice to the user.

**Honest scope**

- **FR-020**: The project MUST maintain a capability register classifying every user-facing capability as **Core** (verified on Windows and macOS), **Extended** (verified individually), or **Unavailable**.
- **FR-021**: Core MUST include: launch; the four ways of opening in FR-012; the viewing actions in FR-013; JPEG, PNG, GIF, WebP, BMP, and TIFF still images; animated GIF and WebP playback; read-only browsing of images inside ZIP and CBZ archives, with the existing protections against oversized, encrypted, and path-escaping archives demonstrated by automated checks; settings; and Korean and English interface text.
- **FR-022**: The application and its documentation MUST expose only Core capabilities and Extended capabilities that have been verified. Unavailable capabilities MUST be hidden or labelled as not yet available.
- **FR-023**: A capability MAY move from Unavailable to Extended only when it has automated checks using real sample files, automated checks for hostile or malformed input, a recorded successful run on both Windows and macOS, and user documentation.
- **FR-024**: Until re-admitted under FR-023, the following are Unavailable: update checking and installation; image formats beyond FR-021; formats decoded by operating-system codecs; file-association and context-menu registration; the thumbnail strip; custom window decoration.
- **FR-025**: While update checking is Unavailable, the application MUST NOT make any network connection.
- **FR-026**: Linux is an experimental target for this release. It MUST continue to build and pass automated checks, and a Linux release file MUST still be provided, labelled experimental and unverified in release notes and documentation. A Linux failure in launch verification MUST be reported but MUST NOT block publication. Linux becomes a supported target only once it passes the same launch verification as Windows and macOS.

**Release control**

- **FR-027**: Before publication, the release process MUST start each release file on its target operating system and confirm that it shows its window, displays a reference image, and exits cleanly. Any failure on a supported target (Windows, macOS) MUST block publication; failures on experimental targets are handled under FR-026. This verification has two layers, both required for a supported target:
  - an **automated launch check** on hosted build machines, run for every release and every proposed change (FR-028);
  - a **clean-machine check**, run manually from a written checklist immediately before each release, on clean Windows 10 and Windows 11 virtual machines and on a real Apple-processor Mac, starting from the downloaded release file exactly as a user would. An Intel-processor Mac is included when one is available; when it is not, the release notes MUST list Intel macOS as unverified (FR-029).
- **FR-028**: Checks run on every proposed change MUST include starting the application and performing the core interactions on Windows and macOS.
- **FR-029**: Release notes MUST state the verified operating systems and processor families, the experimental ones, and known limitations.
- **FR-030**: A unit of work MUST NOT be recorded as complete unless its behaviour has been observed in the started application on each operating system it claims to support, or the gap is recorded explicitly.

**Auditability**

- **FR-031**: Before decoding, the application MUST positively identify a file as a supported image kind from its content, and MUST refuse files that are not identified, independent of file name.
- **FR-032**: Size limits on input MUST be enforced before the content is loaded into memory.
- **FR-033**: Every documented safety property MUST correspond to an automated check that demonstrates it. Properties without such a check MUST be removed from the documentation until one exists.
- **FR-034**: Every third-party component in the shipped product MUST be linked in a published inventory to at least one Core or verified Extended capability. Components serving only Unavailable capabilities MUST NOT be shipped.
- **FR-035**: The release process MUST check bundled components against publicly known vulnerabilities and block publication on unresolved findings unless an acceptance is recorded with a reason.
- **FR-036**: The rules that map user input to viewer behaviour MUST be verifiable by automated checks that do not require a screen.
- **FR-037**: Capabilities MUST NOT be represented in the product by placeholders that do nothing. A capability is either implemented and verified, or absent.

### Key Entities

- **Release File**: A downloadable package for one operating system and processor family, with its version, integrity status, and trust status (publisher-verified or not).
- **Launch Verification Record**: The evidence, per release file, that it was started on its target, showed its window, displayed the reference image, and exited cleanly; includes date, environment, whether the run was the automated launch check or the clean-machine check, and who performed a manual run.
- **Diagnostic Record**: The per-launch account of version, environment, chosen display method, and fatal errors that a user can find and share.
- **Capability Register**: The list of user-facing capabilities, each with its tier (Core, Extended, Unavailable), the evidence supporting that tier, and the operating systems verified.
- **Component Inventory**: The list of bundled third-party components, each with the capability that justifies it and its vulnerability status.
- **First-Launch Guide**: The bilingual instructions that take a user from download to a visible window, including each trust confirmation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On clean installations of Windows 10, Windows 11, and an Apple-processor Mac — plus an Intel-processor Mac when one is available — a first-time user reaches a visible viewer window within 2 minutes of finishing the download, following only the published instructions, in 100 % of the environments checked. Any of the four environments not checked is listed as unverified in the release notes.
- **SC-002**: Across those environments, the operating system reports the application as damaged, corrupt, or missing a required component zero times.
- **SC-003**: For every startup failure condition in the test catalogue (at minimum: no usable display method, unwritable settings location, damaged settings, missing launch file), 100 % produce a user-visible explanation within 5 seconds and a diagnostic record; silent exits number zero.
- **SC-004**: 100 % of on-screen controls in the main window and in every secondary window respond to the pointer alone within 100 ms, measured over the full control checklist on both operating systems.
- **SC-005**: A scripted session (open a folder of 100 images, step through 20, zoom and pan, change fit mode, rotate, enter and leave fullscreen, change one setting, exit) is completed with 100 % success using the mouse only and again using the keyboard only, on Windows and on macOS.
- **SC-006**: Opening a folder of 10,000 images shows the first image within 1 second, and the interface continues to respond to input within 100 ms throughout.
- **SC-007**: Zero releases are published without a passing Launch Verification Record for every operating system and processor family the release lists as supported.
- **SC-008**: 100 % of capabilities reachable in the application or claimed in the documentation appear in the Capability Register as Core or verified Extended; advertised-but-non-working capabilities number zero.
- **SC-009**: 100 % of documented safety properties map to a passing automated check, and 100 % of hostile-input cases in the test catalogue are refused before decoding.
- **SC-010**: 100 % of shipped third-party components are justified in the Component Inventory, and the number of components the product ships is at least 20 % lower than the 2026-11-16 baseline of 446 (at most 356), both counted the same way: unique packages, including the project's own crates, that the shipped application includes across its four shipped desktop targets (Windows x64, macOS arm64 and x86_64, Linux x64), excluding development-, test-, and build-only packages.
- **SC-011**: After this feature ships, the next user report of "the application does not start" can be classified to a cause category from the diagnostic record alone in 100 % of cases where a record was written.
- **SC-012**: In the Windows clean-machine checks, zero sessions show an auxiliary console window, and the operating system marks the main window as not responding zero times.

## Assumptions

- **What "does not run" means**: The report is taken to concern the published `v1.1.0-alpha.1` downloads. A build made from source on the review Mac started and showed its window. The reporter's exact macOS symptom was not provided; the review reproduced a package that fails the operating system's integrity check.
- **Windows symptom is user-reported, not reproduced**: The user reports that the main window appears but does not respond, a console window opens alongside it, and closing the console closes the application. This matches review findings `B6` (console attached to the application) and `A4`–`A7` (controls not responding), and rules out a trust block or missing system component as the primary cause. It does not yet distinguish between controls that never repaint and a window the operating system marks as hung; planning must reproduce on real Windows 10 and 11 and determine which, and User Story 3 scenario 9 and SC-012 cover both.
- **No signing credentials (confirmed)**: The project holds no paid publisher identity for either operating system for this release. Until it does, users will see a one-time trust confirmation, which FR-004 documents. Under strict operating-system security policies that forbid unrecognised publishers outright, launch will remain blocked until FR-005 is exercised with real credentials; this is a known residual limitation.
- **Priority of platforms (confirmed)**: Windows and macOS are the supported platforms this feature must make work. Linux remains in the project as experimental (FR-026). Because the constitution previously listed Linux as an equal target, its Target Platforms section was amended to v2.3.0 (Supported / Experimental tiers) on 2026-11-16 to match.
- **Scope reduction is temporary and reversible**: Capabilities listed in FR-024 are withdrawn from the shipped product, not deleted from the project's ambitions. Their existing work may be retained outside the shipped product and returns through FR-023. Archive viewing is not withdrawn; it is part of Core.
- **Technology decisions belong to planning**: This specification states outcomes. Whether to keep, replace, or repair the present user-interface foundation, and which components to remove, will be decided in `/speckit-plan` using the evidence in the feasibility review, with sources cited as the constitution requires.
- **Performance targets**: The numeric speed and memory targets of specification 002 remain goals. They are not release-blocking for this feature, because no end-to-end way of measuring them exists yet. SC-004 and SC-006 are the responsiveness bars for this feature.
- **Settings continuity**: Existing user settings and their storage location are preserved.
- **Supersession**: Where this specification conflicts with the parity scope of specification 002, this one prevails for the next release. Specification 002's "complete" status is treated as describing library logic only.
- **Governance**: Constitution Principles I–VIII continue to apply, including tests written first, no placeholder production paths, evidence-backed decisions, and explicit disclosure of anything not verified on a given operating system.
- **Verification environments (confirmed)**: Hosted build machines are not clean: they carry development tools and runtimes that ordinary user machines lack, so passing the automated launch check alone does not prove FR-001. Clean Windows 10 and Windows 11 virtual machines and a real Apple-processor Mac are assumed obtainable for the manual clean-machine check (FR-027). An Intel-processor Mac may not be; if it cannot be obtained, that target is listed as unverified under FR-029 rather than assumed to work.
