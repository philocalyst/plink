use anyhow::Result;
use clap::Parser;
use plink::{CleaningOptions, UrlCleaner};

/// Simple URL cleaner CLI
#[derive(Debug, Parser)]
#[command(
    name = "plink",
    about = "Clean URL's by peeling away tracking parameters and other junk"
)]
struct Cli {
    /// Do NOT skip localhost URLs
    #[arg(long)]
    no_skip_localhost: bool,

    /// Do NOT apply referral-marketing rules
    #[arg(long)]
    no_referral_marketing: bool,

    /// Do NOT enable domain blocking
    #[arg(long)]
    no_domain_blocking: bool,

    /// Comma-separated list of blacklisted domains
    #[arg(long, value_name = "DOMAINS")]
    blacklist: Option<String>,

    /// Comma-separated list of additional blocked params
    #[arg(long, value_name = "PARAMS")]
    additional_params: Option<String>,

    /// One or more URLs to clean
    #[arg(value_name = "URL", required = true)]
    urls: Vec<String>,
}

fn parse_csv(input: Option<&str>) -> Vec<String> {
    input
        .map(|s| {
            s.split(',')
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let options = CleaningOptions {
        skip_localhost: !cli.no_skip_localhost,
        apply_referral_marketing: !cli.no_referral_marketing,
        domain_blocking: !cli.no_domain_blocking,
        additional_blocked_params: parse_csv(cli.additional_params.as_deref()),
        blacklisted_domains: parse_csv(cli.blacklist.as_deref()),
    };

    // load the embedded JSON config
    let config_json = include_str!("/Users/philocalyst/Projects/plink/src/data.json");
    let cleaner = UrlCleaner::from_json(config_json, options)?;

    for url in cli.urls {
        match cleaner.clean_url(&url) {
            Ok(result) => {
                // Print the cleaned URL
                println!("{}", result.url);
            }
            Err(e) => {
                eprintln!("error cleaning {}: {}", url, e);
            }
        }
    }

    Ok(())
}
