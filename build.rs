use std::{collections::HashMap, env, fs, path::Path};

use serde::{Deserialize, Serialize};

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
    // The bincode config
    let config = bincode::config::standard();

    // Tell Cargo to re-run build.rs if data.json changes
    println!("cargo:rerun-if-changed=data.json");

    // Load the JSON from the project root
    let json = fs::read_to_string("./Rules/data.min.json")?;

    // Deserialize it
    let url_config: ClearUrlsConfig =
        serde_json::from_str(&json).expect("Failed to parse data.json");

    // Then serialize it into bitcode using serde
    let output = bincode::serde::encode_to_vec(&url_config, config)?;

    let out_dir = env::var("OUT_DIR")?;
    let destination = Path::new(&out_dir).join("data.bin");

    // Write into OUT_DIR data.bin
    fs::write(&destination, output)?;

    Ok(())
}
