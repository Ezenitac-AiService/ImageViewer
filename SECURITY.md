# Security Policy

## Scope

This security policy applies to **Ezenitac-AiService/ImageViewer** (the cross-platform Rust edition).

## Reporting a Vulnerability

If you discover a security vulnerability (such as path traversal, zip-slip, unsafe symlink handling, integer overflow in image decoding, or update-feed spoofing), please report it privately rather than opening a public issue:

- Use **GitHub Private Vulnerability Reporting** (`Security` -> `Advisories` -> `Report a vulnerability`) on `https://github.com/Ezenitac-AiService/ImageViewer`.
- Please include:
  - Affected OS, architecture, and `ImageViewer` version
  - Minimal reproduction steps or sample trigger file (without sensitive personal data)
  - Expected vs. actual behavior

We aim to acknowledge reports within 72 hours and coordinate a fix before public disclosure.

> Note: Please do not report cross-platform Rust vulnerabilities to the upstream `edith2580/ImageViewer` repository.
