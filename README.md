
# Welcome to Plink

Plink is a Rust library and command-line tool for cleaning URLs by removing tracking parameters, referral codes, raw fragments and unwanted redirects. Whether you’re building a web service or just want a quick way to sanitize links in your shell, Plink has you covered.

Get started by installing Plink → see [Install](#install)

---

## Tutorial

### As a CLI

Use Plink to clean one or more URLs right from your terminal:

```shell
plink https://example.com/page?utm_source=newsletter&fbclid=XYZ123
https://example.com/page
```

Pass multiple URLs:

```shell
plink https://site.com/?a=1&utm_medium=email, https://foo.org/?gclid=ABC
https://site.com/?a=1
https://foo.org/
```

Control options:

```shell
plink --no-referral-marketing --additional-params=ref,src \
    https://mysite.com/?src=test&ref=home&utm_campaign=spring
https://mysite.com/
```

### As a library

Add Plink to your `Cargo.toml`:

```toml
[dependencies]
plink = "0.2.2"
```

Then in code:

```rust
use plink::{CleaningOptions, UrlCleaner};

fn main() -> Result<(), Box<dyn Error>> {
    // Build default options (skip localhost, block domains, strip referrals)
    let options = CleaningOptions::default();
    // Load embedded config and compile regexes at startup
    let cleaner = UrlCleaner::from_data(options)?;
    // Clean a URL
    let result = cleaner.clean_url("example.com/?utm_source=foo&gclid=123")?;
    assert_eq!(result.url.as_str(), "https://example.com/");
    Ok(())
}
```

If you need custom options:

```rust
let options = CleaningOptions {
    skip_localhost: false,
    apply_referral_marketing: false,
    domain_blocking: true,
    additional_blocked_params: vec!["fbclid".into(), "gclid".into()],
    blacklisted_domains: vec!["internal.local".into()],
};
let cleaner = UrlCleaner::from_data(options)?;
```

---

## Building and Debugging

Clone the repo and run:

```shell
git clone https://github.com/yourorg/plink.git
cd plink
cargo build --release
```

Run tests:

```shell
cargo test
```

Enable TRACE-level logs and span events:

```shell
RUST_LOG=trace cargo run -- https://example.com/?utm_source=test
```

---

## Install

### From crates.io

```shell
cargo install plink
```

### As a library

```diff
[dependencies]
+ plink = "0.2.3"
```

---

## Changelog

See [CHANGELOG.md] for all notable changes and version history.

---

## Libraries Used

- regex – fast regular expressions  
- url – URL parsing and manipulation  
- serde, serde_json – config serialization  
- bincode – build-time config blob  
- bitcode – optional alternative serialization  
- clap – command-line parsing  
- log, env_logger – logging  
- tracing, tracing-subscriber – structured diagnostics  
- anyhow – error handling  
- urlencoding – percent-decode redirect targets  

---

## Acknowledgements

Inspired by the [ClearURLs] browser extension and its community-maintained ruleset by Kevin Roebert. Thanks to all upstream crate authors!

---

## License

Plink is released under the [MIT License]. See the `LICENSE` file for details.

[CHANGELOG.md]: ./CHANGELOG.md  
[ClearURLs]: https://github.com/ClearURLs/  
[MIT License]: https://opensource.org/licenses/MIT
