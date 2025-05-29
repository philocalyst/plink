use anyhow::{Context, Result};
use log::{debug, info, warn};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use url::Url;

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

/// Compiled provider with regex patterns for performance
#[derive(Debug)]
struct CompiledProvider {
    name: String,
    url_pattern: Regex,
    rules: Vec<Regex>,
    raw_rules: Vec<Regex>,
    exceptions: Vec<Regex>,
    redirections: Vec<Regex>,
    referral_marketing: Vec<Regex>,
    complete_provider: bool,
    force_redirection: bool,
}

/// Result of URL cleaning operation
#[derive(Debug, Clone)]
pub struct CleaningResult {
    /// The cleaned URL
    pub url: Url,
    /// Whether any changes were made
    pub changed: bool,
    /// Whether this should be a redirect
    pub redirect: bool,
    /// Whether the request should be cancelled/blocked
    pub cancel: bool,
    /// Which rules were applied
    pub applied_rules: Vec<String>,
}

/// Configuration options for URL cleaning
#[derive(Debug, Clone)]
pub struct CleaningOptions {
    /// Whether to skip localhost URLs
    pub skip_localhost: bool,
    /// Whether to apply referral marketing rules
    pub apply_referral_marketing: bool,
    /// Whether to enable domain blocking
    pub domain_blocking: bool,
    /// Additional blocked parameters (like the neat_url approach)
    pub additional_blocked_params: Vec<String>,
    /// Domains to exclude from cleaning
    pub blacklisted_domains: Vec<String>,
}

impl Default for CleaningOptions {
    fn default() -> Self {
        Self {
            skip_localhost: true,
            apply_referral_marketing: true,
            domain_blocking: true,
            additional_blocked_params: Vec::new(),
            blacklisted_domains: Vec::new(),
        }
    }
}

/// Main URL cleaner that applies rules to sanitize URLs
pub struct UrlCleaner {
    providers: Vec<CompiledProvider>,
    options: CleaningOptions,
}

impl UrlCleaner {
    /// Create a new URL cleaner from configuration
    pub fn new(config: ClearUrlsConfig, options: CleaningOptions) -> Result<Self> {
        info!(
            "Initializing URL cleaner with {} providers",
            config.providers.len()
        );

        let mut providers = Vec::new();

        for (name, provider) in config.providers {
            match Self::compile_provider(name.clone(), provider) {
                Ok(compiled) => providers.push(compiled),
                Err(e) => {
                    warn!("Failed to compile provider '{}': {}", name, e);
                    continue;
                }
            }
        }

        info!("Successfully compiled {} providers", providers.len());

        Ok(Self { providers, options })
    }

    /// Load configuration from JSON string
    pub fn from_json(json: &str, options: CleaningOptions) -> Result<Self> {
        let config: ClearUrlsConfig =
            serde_json::from_str(json).context("Failed to parse configuration JSON")?;
        Self::new(config, options)
    }

    /// Clean a URL by removing tracking parameters
    pub fn clean_url(&self, url: &str) -> Result<CleaningResult> {
        // We need to make this owned for the base manipulation
        let mut url = url.to_string();

        // Add the boilerplate if it's not present
        if !url.contains("https://") {
            url = format!("https://{}", url);
        } else if !url.contains("http://") {
            url = format!("http://{}", url);
        }

        let mut url = Url::parse(&url).context("Failed to parse URL")?;

        debug!("Cleaning URL: {}", url);

        // Check if we should skip this URL
        if self.should_skip_url(&url) {
            debug!("Skipping URL due to configuration: {}", url);
            return Ok(CleaningResult {
                url,
                changed: false,
                redirect: false,
                cancel: false,
                applied_rules: Vec::new(),
            });
        }

        let original_url = url.clone();
        let mut changed = false;
        let mut applied_rules = Vec::new();

        // Apply provider-specific rules
        for provider in &self.providers {
            if provider.matches_url(&url)? && !provider.matches_exception(&url)? {
                applied_rules.push(provider.name.clone());
                let result = self.apply_provider_rules(provider, &mut url)?;

                if result.redirect {
                    info!(
                        "URL {} redirected by provider {}",
                        original_url, provider.name
                    );
                    return Ok(CleaningResult {
                        url,
                        changed: true,
                        redirect: true,
                        cancel: false,
                        applied_rules,
                    });
                }

                if result.cancel {
                    info!("URL {} blocked by provider {}", original_url, provider.name);
                    return Ok(CleaningResult {
                        url,
                        changed: false,
                        redirect: false,
                        cancel: true,
                        applied_rules,
                    });
                }

                if result.changed {
                    changed = true;
                    applied_rules.extend(result.applied_rules);
                }
            }
        }

        // Apply additional blocked parameters (neat_url style)
        if self.apply_additional_param_rules(&mut url)? {
            changed = true;
            applied_rules.push("additional_params".to_string());
        }

        if changed {
            info!("Cleaned URL: {} -> {}", original_url, url);
        } else {
            debug!("No changes made to URL: {}", url);
        }

        Ok(CleaningResult {
            url,
            changed,
            redirect: false,
            cancel: false,
            applied_rules,
        })
    }

    fn compile_provider(name: String, provider: Provider) -> Result<CompiledProvider> {
        let url_pattern = Regex::new(&provider.url_pattern)
            .context(format!("Invalid URL pattern for provider {}", name))?;

        // Append the rules verbatim
        let rules = provider
            .rules
            .iter()
            .map(|r| Regex::new(r))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to compile rules")?;

        let raw_rules = provider
            .raw_rules
            .iter()
            .map(|r| Regex::new(r))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to compile raw rules")?;

        let exceptions = provider
            .exceptions
            .iter()
            .map(|r| Regex::new(r))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to compile exceptions")?;

        let redirections = provider
            .redirections
            .iter()
            .map(|r| Regex::new(r))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to compile redirections")?;

        let referral_marketing = provider
            .referral_marketing
            .iter()
            .map(|r| Regex::new(&format!("^{}$", r)))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to compile referral marketing rules")?;

        Ok(CompiledProvider {
            name,
            url_pattern,
            rules,
            raw_rules,
            exceptions,
            redirections,
            referral_marketing,
            complete_provider: provider.complete_provider,
            force_redirection: provider.force_redirection,
        })
    }

    fn should_skip_url(&self, url: &Url) -> bool {
        // Skip localhost if configured
        if self.options.skip_localhost && self.is_localhost(url) {
            return true;
        }

        // Skip blacklisted domains
        if let Some(host) = url.host_str() {
            for blacklisted in &self.options.blacklisted_domains {
                if host.ends_with(blacklisted) {
                    return true;
                }
            }
        }

        false
    }

    fn is_localhost(&self, url: &Url) -> bool {
        if let Some(host) = url.host_str() {
            host == "localhost"
                || host.starts_with("127.")
                || host.starts_with("192.168.")
                || host.starts_with("10.")
                || host.starts_with("172.")
        } else {
            false
        }
    }

    fn apply_provider_rules(
        &self,
        provider: &CompiledProvider,
        url: &mut Url,
    ) -> Result<CleaningResult> {
        let mut changed = false;
        let mut applied_rules = Vec::new();

        // Check for cancellation (complete provider blocking)
        if provider.complete_provider && self.options.domain_blocking {
            return Ok(CleaningResult {
                url: url.clone(),
                changed: false,
                redirect: false,
                cancel: true,
                applied_rules: vec![provider.name.clone()],
            });
        }

        // Check for redirections
        if let Some(redirect_url) = self.apply_redirections(provider, url)? {
            *url = redirect_url;
            return Ok(CleaningResult {
                url: url.clone(),
                changed: true,
                redirect: true,
                cancel: false,
                applied_rules: vec![format!("{}_redirect", provider.name)],
            });
        }

        // Apply raw rules (regex replacements on the entire URL)
        for (i, raw_rule) in provider.raw_rules.iter().enumerate() {
            let original = url.to_string();
            let cleaned = raw_rule.replace_all(&original, "");
            if cleaned != original {
                *url = Url::parse(&cleaned).context("Invalid URL after applying raw rule")?;
                changed = true;
                applied_rules.push(format!("{}_raw_{}", provider.name, i));
                debug!("Applied raw rule {} to {}", i, provider.name);
            }
        }

        // Apply parameter rules
        if self.apply_parameter_rules(provider, url)? {
            changed = true;
        }

        Ok(CleaningResult {
            url: url.clone(),
            changed,
            redirect: false,
            cancel: false,
            applied_rules,
        })
    }

    fn apply_redirections(&self, provider: &CompiledProvider, url: &Url) -> Result<Option<Url>> {
        for redirection in &provider.redirections {
            if let Some(captures) = redirection.captures(url.as_str()) {
                if let Some(redirect_match) = captures.get(1) {
                    let decoded_url = urlencoding::decode(redirect_match.as_str())
                        .context("Failed to decode redirect URL")?;
                    let redirect_url = Url::parse(&decoded_url).context("Invalid redirect URL")?;
                    debug!("Found redirection: {} -> {}", url, redirect_url);
                    return Ok(Some(redirect_url));
                }
            }
        }
        Ok(None)
    }

    fn apply_parameter_rules(&self, provider: &CompiledProvider, url: &mut Url) -> Result<bool> {
        let mut changed = false;

        // Collect all rules to apply
        let mut all_rules = provider.rules.clone();
        if self.options.apply_referral_marketing {
            all_rules.extend(provider.referral_marketing.clone());
        }

        // Remove matching parameters.
        // We only need the key, because that's what the dataset is based on.
        let params_to_remove: Vec<String> = url
            .query_pairs()
            .filter_map(|(key, _)| {
                for rule in &all_rules {
                    // Match verbatim keys
                    let rule = RegexBuilder::new(&format!("^{}$", rule))
                        .case_insensitive(true)
                        .build().expect("We're taking an existing regex and making it only match verbatim, shouldn't fail.");

                    if rule.is_match(&key) {
                        debug!(
                            "Parameter '{}' matches rule in provider {}",
                            key, provider.name
                        );
                        return Some(key.to_string());
                    }
                }
                None
            })
            .collect();

        if !params_to_remove.is_empty() {
            // Then we use the key to get the full query to remove on
            let new_params = url
                .query_pairs()
                .filter(|(key, _)| !params_to_remove.contains(&key.to_string()))
                .collect::<Vec<_>>();

            // Rebuild query string
            if new_params.is_empty() {
                // If we do all of the matching and replacement, and we end up with no params, than the url is free of them.
                url.set_query(None);
            } else {
                // Otherwise we're free to rebuild the string, which comes down to replicating the param format as we're rebuilding
                let query_string = new_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                url.set_query(Some(&query_string));
            }

            changed = true;
        }

        Ok(changed)
    }

    fn apply_additional_param_rules(&self, url: &mut Url) -> Result<bool> {
        if self.options.additional_blocked_params.is_empty() {
            return Ok(false);
        }

        // Refer to apply_param_rules for notes on this logic
        let params_to_remove: HashSet<String> = self
            .options
            .additional_blocked_params
            .iter()
            .cloned()
            .collect();

        let original_count = url.query_pairs().count();

        let new_params: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(key, _)| !params_to_remove.contains(key.as_ref()))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        if new_params.len() != original_count {
            if new_params.is_empty() {
                url.set_query(None);
            } else {
                let query_string = new_params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                url.set_query(Some(&query_string));
            }
            return Ok(true);
        }

        Ok(false)
    }
}

impl CompiledProvider {
    fn matches_url(&self, url: &Url) -> Result<bool> {
        Ok(self.url_pattern.is_match(url.as_str()))
    }

    fn matches_exception(&self, url: &Url) -> Result<bool> {
        for exception in &self.exceptions {
            if exception.is_match(url.as_str()) {
                debug!("URL {} matches exception in provider {}", url, self.name);
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cleaning() {
        let config = r#"
        {
            "providers": {
                "Google": {
                    "urlPattern": ".*google\\.com.*",
                    "rules": ["utm_source", "utm_medium", "utm_campaign"]
                }
            }
        }"#;

        let cleaner = UrlCleaner::from_json(config, CleaningOptions::default()).unwrap();
        let result = cleaner
            .clean_url("https://google.com/search?q=test&utm_source=newsletter")
            .unwrap();

        assert!(result.changed);
        assert_eq!(result.url.as_str(), "https://google.com/search?q=test");
    }

    #[test]
    fn test_additional_params() {
        let config = r#"{"providers": {}}"#;
        let options = CleaningOptions {
            additional_blocked_params: vec!["fbclid".to_string(), "gclid".to_string()],
            ..Default::default()
        };

        let cleaner = UrlCleaner::from_json(config, options).unwrap();
        let result = cleaner
            .clean_url("https://example.com/?test=1&fbclid=123&gclid=456")
            .unwrap();

        assert!(result.changed);
        assert_eq!(result.url.as_str(), "https://example.com/?test=1");
    }
}
