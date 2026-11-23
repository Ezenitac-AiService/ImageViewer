# Contract: Input Acceptance (Formats, Sizes, Archives)

**Covers**: FR-021, FR-024, FR-031–FR-033; User Story 5 scenario 3, User Story 6 scenarios 1–2 |
**Research**: R8, R9

## 1. Core Format Allowlist

`FormatGuard::identify(bytes) -> Option<CoreFormat>` inspects leading bytes only.

| CoreFormat | Signature (offset 0 unless noted) |
|-----------|-----------------------------------|
| `Jpeg` | `FF D8 FF` |
| `Png` | `89 50 4E 47 0D 0A 1A 0A` |
| `Gif` | ASCII `GIF87a` or `GIF89a` |
| `WebP` | ASCII `RIFF` at 0 and `WEBP` at 8 |
| `Bmp` | ASCII `BM` (also requires ≥ 26 bytes and a DIB header size ∈ {12, 40, 52, 56, 108, 124} at offset 14) |
| `Tiff` | `49 49 2A 00` or `4D 4D 00 2A` |
| `Zip` | `50 4B 03 04` or `50 4B 05 06` |

**Rules**:

1. Order: denylist (existing `validate_header`) → `identify` → decode. Both layers run.
2. The decoder is selected by `CoreFormat`, never by extension. A `.jpg` file containing PNG bytes
   is decoded as PNG.
3. `None` → error `error.unsupported_format` ("This format is not supported in this version"),
   shown as a status message. No decoder is invoked.
4. With the `extended-formats` feature, `identify` gains additional entries. That table is out of
   scope for the shipped build.

## 2. Size and Resource Caps (enforced before allocation)

| Cap | Value | Enforced at |
|-----|-------|-------------|
| Single file read | 512 MB | `FolderSource::read_entry_bytes`: `metadata.len()` check, then `take(cap + 1)` |
| Pixels | `max_image_megapixels` (default 150, clamp 10–1000) | Header dimensions, before decode (existing) |
| ZIP entry uncompressed | `max_zip_entry_megabytes` (existing) | Streaming read (existing) |
| ZIP entries / central directory / container | 50,000 / 64 MB / 4 GB | `ZipSource::open` (spec 002 clarification; verify it is implemented, else add) |
| Animation stream | 512 MB total, 128 MB forward ring | Existing |

Exceeding any cap produces the existing localized "unsafe or malformed" message. Nothing is
allocated beyond the cap.

## 3. Archives (Core)

- Read-only. Encrypted entries are refused with `error.encrypted_archive`.
- Entry names that are absolute, contain `..`, or resolve outside the archive root are skipped and
  logged (Zip Slip).
- Entries inside an archive are subject to §1 and §2 individually.

## 4. Negative Tests Required (real fixtures, no mocks)

| # | Input | Expected |
|---|-------|----------|
| 1 | SVG / HTML / PDF / PS bytes renamed `.png` | Refused by the denylist |
| 2 | 1 KB of random bytes named `.jpg` | `unsupported_format`; decoder not called |
| 3 | A valid QOI file named `.qoi` (extended format) | `unsupported_format` in the shipped build |
| 4 | A valid PNG named `.jpg` | Decodes as PNG |
| 5 | A sparse file whose metadata length is cap + 1 | Refused before read; peak RSS increase < 10 MB |
| 6 | A PNG declaring 100,000 × 100,000 | `ExceedsPixelLimit` before allocation |
| 7 | A ZIP with a `../evil.png` entry | Entry skipped; others readable |
| 8 | A ZIP bomb (small container, entry over the cap) | Refused at the cap |
| 9 | An encrypted ZIP | `encrypted_archive` |
| 10 | A ZIP with 50,001 entries | Refused at open |
| 11 | BMP magic with a truncated header | `unsupported_format` |

Each documented safety property in README / SECURITY maps to one row here or in the launch
contract §6. Properties with no row are removed from the documentation (FR-033).
