# Changelog

All notable changes to this project are documented in this file.  
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] – 2025-05-29

### Added
- Verbatim, case-insensitive parameter-key matching for query-parameter removal

### Changed
- Use the README’s example URLs in `src/main.rs`
- Simplify `applied_rules` accumulation in `UrlCleaner` to collect provider names rather than reset per rule

### Removed
- Compile-time full-string regex anchoring for `provider.rules`; patterns are now used as-provided
- The regex-based URL-transformation loop for `provider.rules` (cleaning now relies on `raw_rules` and parameter rules only)

[Unreleased]: https://github.com/yourorg/yourrepo/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourorg/yourrepo/compare/...v0.1.0
