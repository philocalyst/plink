# Changelog

All notable changes to this project are documented in this file.  
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] – 2025-06-19

### Added
- **Tracing support for debugging**: Added comprehensive tracing infrastructure using the `tracing` and `tracing-subscriber` crates to enable detailed debugging and observability
  - Instrumented key functions (`from_data`, `clean_url`) with `#[instrument]` attributes
  - Configured tracing subscriber with TRACE level logging and span events for function entry/exit
  - Handles edge cases and provides detailed execution flow visibility
  - Added bincode configuration and encoding/decoding capabilities in build process
  - Maintains compatibility with existing bitcode serialization
- **Enhanced debugging capabilities**: Added `Debug` trait implementation for `UrlCleaner` struct to improve development experience

### Changed
- **Bincode serialization support**: Integrated the `bincode` crate as a replacement serialization option over bitcode (seems quicker, more maintained)
- **Build process improvements**: Updated build script to use bincode for data serialization while maintaining bitcode compatibility
- **Dependency updates**: Added new dependencies for tracing (`tracing`, `tracing-subscriber`, `nu-ansi-term`, `once_cell`, `sharded-slab`, `thread_local`) and serialization (`bincode`, `bincode_derive`, `unty`, `virtue`)

## [0.2.1] – 2025-06-15

### Added
- Compile‐time serialization of `data.json` into a Bitcode blob (`data.bin`) via a new `build.rs`.
- Embed the compiled blob at runtime with  
  `include_bytes!(concat!(env!("OUT_DIR"), "/data.bin"))` and deserialize it using `bitcode::deserialize`.
- Introduce `UrlCleaner::from_data(options: CleaningOptions)` to construct a cleaner from the embedded configuration.
- Add `bitcode` (with `serde` feature), `bitcode_derive`, `bytemuck` and `glam` as dependencies, plus `serde` and `serde_json` as build-dependencies.

### Changed
- Remove JSON parsing at runtime; drop `include_str!("data.json")` and the obsolete `UrlCleaner::from_json` method.
- Update `src/main.rs`, library code and tests to use `from_data` instead of `from_json`.
- Move `data.json` from `src/` to the project root so it can be picked up by the build script.
- Refine URL‐scheme normalization to only prepend `https://` or `http://` when the input does not already start with those prefixes.
- Ensure embedded data is loaded from the `OUT_DIR` environment variable for greater portability.
- Bump `Cargo.toml` and `Cargo.lock` to include the new Bitcode-related crates.

### Removed
- The `UrlCleaner::from_json(&str, CleaningOptions)` constructor.

## [0.2.0] – 2025-05-29

### Added
- Full CLI interface via `clap`  
  • Flags: `--no-skip-localhost`, `--no-referral-marketing`,  
    `--no-domain-blocking`, `--blacklist=<DOMAINS>`,  
    `--additional-params=<PARAMS>`  
  • Positional `URL…` arguments
- `parse_csv` helper to turn comma-separated strings into `Vec<String>`
- Automatic `https://` prefix for URLs that lack a scheme
- Introduced `clap` (derive) dependency; updated `Cargo.toml` and `Cargo.lock`

### Changed
- Switched sample URLs in `src/main.rs` to match the README examples
- Refactored `src/main.rs` to build `CleaningOptions` from CLI flags
- Moved loading of the embedded JSON config to after CLI parsing

## [0.1.0] – 2025-05-29

### Added
- Verbatim, case-insensitive parameter-key matching for query-parameter removal

### Changed
- Use the README’s example URLs in `src/main.rs`
- Simplify `applied_rules` accumulation in `UrlCleaner` to collect provider names

### Removed
- Compile-time full-string regex anchoring for `provider.rules`; patterns are now
  used as-provided
- The regex-based URL-transformation loop for `provider.rules` (cleaning now relies
  on `raw_rules` and parameter rules only)

## [0.1.0] – 2025-05-29

### Added
- Verbatim, case-insensitive parameter-key matching for query-parameter removal

### Changed
- Use the README’s example URLs in `src/main.rs`
- Simplify `applied_rules` accumulation in `UrlCleaner` to collect provider names rather than reset per rule

### Removed
- Compile-time full-string regex anchoring for `provider.rules`; patterns are now used as-provided
- The regex-based URL-transformation loop for `provider.rules` (cleaning now relies on `raw_rules` and parameter rules only)

[Unreleased]: https://github.com/yourorg/yourrepo/compare/v0.2.1...HEAD  
[0.2.1]: https://github.com/yourproject/plink/compare/v0.2.0...v0.2.1
[0.2.0]:     https://github.com/yourorg/yourrepo/compare/v0.1.0...v0.2.0  
[0.1.0]: https://github.com/yourorg/yourrepo/compare/...v0.1.0
