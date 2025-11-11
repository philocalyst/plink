use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::Path};

/// Configuration for URL cleaning rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearUrlsConfig {
    pub providers: HashMap<String, Provider>,
}

/// A provider defines cleaning rules for specific domains/services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    #[serde(rename = "urlPattern")]
    pub url_pattern: String,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(rename = "rawRules", default)]
    pub raw_rules: Vec<String>,
    #[serde(default)]
    pub exceptions: Vec<String>,
    #[serde(default)]
    pub redirections: Vec<String>,
    #[serde(rename = "referralMarketing", default)]
    pub referral_marketing: Vec<String>,
    #[serde(rename = "completeProvider", default)]
    pub complete_provider: bool,
    #[serde(rename = "forceRedirection", default)]
    pub force_redirection: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=Rules/data.min.json");

    let json = fs::read_to_string("./Rules/data.min.json")?;
    let url_config: ClearUrlsConfig = serde_json::from_str(&json)?;

    // Track unique regex patterns for deduplication
    let mut regex_counter = 0;
    let mut regex_map: HashMap<String, proc_macro2::Ident> = HashMap::new();

    let mut get_regex_name = |pattern: &str| -> proc_macro2::Ident {
        if let Some(name) = regex_map.get(pattern) {
            return name.clone();
        }
        let name = format_ident!("RE_{}", regex_counter.to_string());
        regex_counter += 1;
        regex_map.insert(pattern.to_string(), name.clone());
        name
    };

    // Collect all unique regex patterns
    for provider in url_config.providers.values() {
        get_regex_name(&provider.url_pattern);
        for rule in &provider.rules {
            get_regex_name(rule);
        }
        for rule in &provider.raw_rules {
            get_regex_name(rule);
        }
        for exc in &provider.exceptions {
            get_regex_name(exc);
        }
        for redir in &provider.redirections {
            get_regex_name(redir);
        }
        for ref_mark in &provider.referral_marketing {
            get_regex_name(ref_mark);
        }
    }

    // Generate all regex statics using LazyLock
    let mut regex_defs = Vec::new();
    for (pattern, name) in &regex_map {
        regex_defs.push(quote! {
            static #name: std::sync::LazyLock<regex::Regex> =
                std::sync::LazyLock::new(|| regex::Regex::new(#pattern).unwrap());
        });
    }

    // Generate provider data
    let mut provider_entries = Vec::new();

    for (provider_name, provider) in &url_config.providers {
        // Create a valid Rust identifier from the provider name
        let mut safe_name = provider_name
            .replace(['.', '-', ' ', '/'], "_")
            .to_uppercase();

        // Rust identifiers can't start with a number, so prefix with P_
        if safe_name
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_digit())
        {
            safe_name = format!("P_{}", safe_name);
        }

        // Remove any remaining invalid characters
        safe_name = safe_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        let safe_ident = format_ident!("{}", safe_name);
        let url_pattern_regex = regex_map.get(&provider.url_pattern).unwrap();

        // Generate rules array
        let rules_array_name = format_ident!("{}_RULES", safe_name);
        let rules: Vec<_> = provider
            .rules
            .iter()
            .map(|r| regex_map.get(r).unwrap())
            .collect();

        let rules_def = if !rules.is_empty() {
            quote! {
                static #rules_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![#(&#rules),*]);
            }
        } else {
            quote! {
                static #rules_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![]);
            }
        };

        // Generate raw_rules array
        let raw_rules_array_name = format_ident!("{}_RAW_RULES", safe_name);
        let raw_rules: Vec<_> = provider
            .raw_rules
            .iter()
            .map(|r| regex_map.get(r).unwrap())
            .collect();

        let raw_rules_def = if !raw_rules.is_empty() {
            quote! {
                static #raw_rules_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![#(&#raw_rules),*]);
            }
        } else {
            quote! {
                static #raw_rules_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![]);
            }
        };

        // Generate exceptions array
        let exceptions_array_name = format_ident!("{}_EXCEPTIONS", safe_name);
        let exceptions: Vec<_> = provider
            .exceptions
            .iter()
            .map(|e| regex_map.get(e).unwrap())
            .collect();

        let exceptions_def = if !exceptions.is_empty() {
            quote! {
                static #exceptions_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![#(&#exceptions),*]);
            }
        } else {
            quote! {
                static #exceptions_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![]);
            }
        };

        // Generate redirections array
        let redirections_array_name = format_ident!("{}_REDIRECTIONS", safe_name);
        let redirections: Vec<_> = provider
            .redirections
            .iter()
            .map(|r| regex_map.get(r).unwrap())
            .collect();

        let redirections_def = if !redirections.is_empty() {
            quote! {
                static #redirections_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![#(&#redirections),*]);
            }
        } else {
            quote! {
                static #redirections_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![]);
            }
        };

        // Generate referral_marketing array
        let referral_array_name = format_ident!("{}_REFERRAL", safe_name);
        let referral_marketing: Vec<_> = provider
            .referral_marketing
            .iter()
            .map(|r| regex_map.get(r).unwrap())
            .collect();

        let referral_def = if !referral_marketing.is_empty() {
            quote! {
                static #referral_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![#(&#referral_marketing),*]);
            }
        } else {
            quote! {
                static #referral_array_name: std::sync::LazyLock<Vec<&'static regex::Regex>> =
                    std::sync::LazyLock::new(|| vec![]);
            }
        };

        let complete_provider = provider.complete_provider;
        let force_redirection = provider.force_redirection;

        provider_entries.push(quote! {
            #rules_def
            #raw_rules_def
            #exceptions_def
            #redirections_def
            #referral_def

            #provider_name => Provider {
                url_pattern: &#url_pattern_regex,
                rules: &#rules_array_name,
                raw_rules: &#raw_rules_array_name,
                exceptions: &#exceptions_array_name,
                redirections: &#redirections_array_name,
                referral_marketing: &#referral_array_name,
                complete_provider: #complete_provider,
                force_redirection: #force_redirection,
            },
        });
    }

    // Generate the complete file
    let output = quote! {
        use regex::Regex;
        use std::sync::LazyLock;

        #[derive(Debug)]
        pub struct Provider {
            pub url_pattern: &'static Regex,
            pub rules: &'static LazyLock<Vec<&'static Regex>>,
            pub raw_rules: &'static LazyLock<Vec<&'static Regex>>,
            pub exceptions: &'static LazyLock<Vec<&'static Regex>>,
            pub redirections: &'static LazyLock<Vec<&'static Regex>>,
            pub referral_marketing: &'static LazyLock<Vec<&'static Regex>>,
            pub complete_provider: bool,
            pub force_redirection: bool,
        }

        // Generate all regex statics
        #(#regex_defs)*

        // Generate provider map
        pub static PROVIDERS: phf::Map<&'static str, Provider> = phf::phf_map! {
            #(#provider_entries)*
        };
    };

    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("generated.rs");
    fs::write(dest_path, output.to_string())?;

    Ok(())
}
