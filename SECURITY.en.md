# Security Policy

**Language / 언어**: [한국어](SECURITY.md) | **[English](SECURITY.en.md)**

## Scope

This security policy applies to **Ezenitac-AiService/ImageViewer** (the cross-platform Rust edition).

## Reporting a Vulnerability

If you discover a security vulnerability (such as path traversal, zip-slip, unsafe symlink handling, integer overflow or excessive memory use in image decoding), please report it privately rather than opening a public issue:

- Use **GitHub Private Vulnerability Reporting** (`Security` -> `Advisories` -> `Report a vulnerability`) on `https://github.com/Ezenitac-AiService/ImageViewer`.
- Please include:
  - Affected OS, architecture, and `ImageViewer` version
  - Minimal reproduction steps or sample trigger file (without sensitive personal data)
  - Expected vs. actual behavior

We aim to acknowledge reports within 72 hours and coordinate a fix before public disclosure.

> Note: Please do not report cross-platform Rust vulnerabilities to the upstream `edith2580/ImageViewer` repository.

## Safety Properties and the Tests That Demonstrate Them

Every safety property stated in this document is demonstrated by the automated tests below. Properties without a test are not documented (specs/004 FR-033). If you find a violation, please use the private reporting process above.

| Property | Test |
|---|---|
| Archive entries pointing outside the archive are skipped (Zip Slip) | `path_security_tests::archive_entry_key_normalization_rejects_traversal_and_symlinks`, `sources_tests::zip_source_enforces_preflight_eocd_caps_traversal_and_symlinks` |
| Symlink entries in archives and symlinks in folders are not opened | `sources_tests::zip_source_enforces_preflight_eocd_caps_traversal_and_symlinks`, `sources_tests::folder_source_natural_sorts_and_skips_symlinks` |
| Archives over 50,000 entries, a 64 MB central directory, or 4 GB are refused | `sources_tests::zip_source_enforces_preflight_eocd_caps_traversal_and_symlinks` |
| Entries over the uncompressed size cap (zip bombs) are refused | `sources_tests::zip_entry_larger_than_the_cap_is_refused_when_read` |
| Encrypted archive entries are refused | `sources_tests::encrypted_zip_entries_are_refused` |
| Script and document content (SVG, HTML, PDF, PostScript) is refused whatever the file is called | `decode_color_resize_tests::script_and_document_payloads_named_png_are_refused` |
| Content not identified as a supported format is not decoded (the extension is ignored) | `format_guard_tests::identify::*`, `decode_color_resize_tests::random_bytes_named_jpg_are_unsupported_without_decoding`, `decode_color_resize_tests::extended_format_signatures_are_unsupported_in_the_default_build` |
| The image size cap (150 MP by default) is checked from the header, before memory is allocated | `decode_color_resize_tests::huge_declared_png_dimensions_fail_before_allocation`, `decode_color_resize_tests::enforces_pixel_cap_and_performs_simd_downscale_and_hdr_tonemap` |
| Files over 512 MB are refused before they are read | `sources_tests::oversized_file_is_refused_before_it_is_read` |
| Settings files over 64 KB or damaged are reset to defaults with a notice | `settings_tests::load_outcome::oversized_file_is_recovered_from_damage`, `settings_tests::load_outcome::malformed_json_is_recovered_from_damage` |
| A symlinked settings file is not read | `settings_tests::load_outcome::symlinked_settings_file_is_unreadable` |
| Concurrent saves from several instances never corrupt the settings file | `settings_tests::load_outcome::concurrent_saves_from_two_instances_never_corrupt_the_file` |
| Diagnostic records never contain full folder paths or line-break control characters | `logger_and_i18n_tests::async_logger_redacts_paths_rotates_and_flushes_on_drop`, `diagnostics_tests::values_never_carry_control_characters_or_host_paths`, `path_security_tests::path_redactor_strips_paths_and_sanitizes_control_chars` |
| The shipped build contains no network client (the app makes no network connections) | `packaging/verify/component-count.sh` (CI) |
| Dependencies with known vulnerabilities block the release | `cargo deny check advisories` (CI·릴리스) |
