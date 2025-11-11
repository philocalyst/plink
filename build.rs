use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, fs, path::Path};

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

fn escape_rust_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=Rules/data.min.json");

    let json = fs::read_to_string("./Rules/data.min.json")?;
    let url_config: ClearUrlsConfig = serde_json::from_str(&json)?;

    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let mut code = String::new();

    // Generate imports
    code.push_str(
        r#"
use regex::Regex;
use lazy_regex::regex;

#[derive(Debug)]
pub struct Provider {
    pub url_pattern: &'static Regex,
    pub rules: &'static [&'static Regex],
    pub raw_rules: &'static [&'static Regex],
    pub exceptions: &'static [&'static Regex],
    pub redirections: &'static [&'static Regex],
    pub referral_marketing: &'static [&'static Regex],
    pub complete_provider: bool,
    pub force_redirection: bool,
}

"#,
    );

    // Track unique regex patterns for deduplication
    let mut regex_counter = 0;
    let mut regex_map: HashMap<String, String> = HashMap::new();

    let mut get_regex_name = |pattern: &str| -> String {
        if let Some(name) = regex_map.get(pattern) {
            return name.clone();
        }
        let name = format!("RE_{}", regex_counter);
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

    // Generate all regex statics using lazy_regex
    for (pattern, name) in &regex_map {
        let escaped = escape_rust_string(pattern);
        code.push_str(&format!(
            "static {}: &Regex = regex!(r\"{}\");\n",
            name, escaped
        ));
    }

    code.push_str("\n");

    // Generate array statics for each provider's rule lists
    let mut provider_data = Vec::new();

    for (provider_name, provider) in &url_config.providers {
        let safe_name = provider_name.replace(['.', '-', ' '], "_").to_uppercase();

        // Generate rules array
        if !provider.rules.is_empty() {
            let array_name = format!("{}_RULES", safe_name);
            code.push_str(&format!("static {}: &[&Regex] = &[\n", array_name));
            for rule in &provider.rules {
                let regex_name = regex_map.get(rule).unwrap();
                code.push_str(&format!("    {},\n", regex_name));
            }
            code.push_str("];\n\n");
        }

        // Generate raw_rules array
        if !provider.raw_rules.is_empty() {
            let array_name = format!("{}_RAW_RULES", safe_name);
            code.push_str(&format!("static {}: &[&Regex] = &[\n", array_name));
            for rule in &provider.raw_rules {
                let regex_name = regex_map.get(rule).unwrap();
                code.push_str(&format!("    {},\n", regex_name));
            }
            code.push_str("];\n\n");
        }

        // Generate exceptions array
        if !provider.exceptions.is_empty() {
            let array_name = format!("{}_EXCEPTIONS", safe_name);
            code.push_str(&format!("static {}: &[&Regex] = &[\n", array_name));
            for exc in &provider.exceptions {
                let regex_name = regex_map.get(exc).unwrap();
                code.push_str(&format!("    {},\n", regex_name));
            }
            code.push_str("];\n\n");
        }

        // Generate redirections array
        if !provider.redirections.is_empty() {
            let array_name = format!("{}_REDIRECTIONS", safe_name);
            code.push_str(&format!("static {}: &[&Regex] = &[\n", array_name));
            for redir in &provider.redirections {
                let regex_name = regex_map.get(redir).unwrap();
                code.push_str(&format!("    {},\n", regex_name));
            }
            code.push_str("];\n\n");
        }

        // Generate referral_marketing array
        if !provider.referral_marketing.is_empty() {
            let array_name = format!("{}_REFERRAL", safe_name);
            code.push_str(&format!("static {}: &[&Regex] = &[\n", array_name));
            for ref_mark in &provider.referral_marketing {
                let regex_name = regex_map.get(ref_mark).unwrap();
                code.push_str(&format!("    {},\n", regex_name));
            }
            code.push_str("];\n\n");
        }

        provider_data.push((provider_name.clone(), safe_name, provider.clone()));
    }

    // Generate the PHF map
    code.push_str("pub static PROVIDERS: phf::Map<&'static str, Provider> = phf::phf_map! {\n");

    for (provider_name, safe_name, provider) in &provider_data {
        let url_pattern_regex = regex_map.get(&provider.url_pattern).unwrap();

        let rules_ref = if !provider.rules.is_empty() {
            format!("{}_RULES", safe_name)
        } else {
            "&[]".to_string()
        };

        let raw_rules_ref = if !provider.raw_rules.is_empty() {
            format!("{}_RAW_RULES", safe_name)
        } else {
            "&[]".to_string()
        };

        let exceptions_ref = if !provider.exceptions.is_empty() {
            format!("{}_EXCEPTIONS", safe_name)
        } else {
            "&[]".to_string()
        };

        let redirections_ref = if !provider.redirections.is_empty() {
            format!("{}_REDIRECTIONS", safe_name)
        } else {
            "&[]".to_string()
        };

        let referral_ref = if !provider.referral_marketing.is_empty() {
            format!("{}_REFERRAL", safe_name)
        } else {
            "&[]".to_string()
        };

        code.push_str(&format!(
            r#"    "{}" => Provider {{
        url_pattern: {},
        rules: {},
        raw_rules: {},
        exceptions: {},
        redirections: {},
        referral_marketing: {},
        complete_provider: {},
        force_redirection: {},
    }},
"#,
            escape_rust_string(provider_name),
            url_pattern_regex,
            rules_ref,
            raw_rules_ref,
            exceptions_ref,
            redirections_ref,
            referral_ref,
            provider.complete_provider,
            provider.force_redirection
        ));
    }

    code.push_str("};\n");

    fs::write(dest_path, code)?;

    Ok(())
}
