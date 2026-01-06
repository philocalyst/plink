use std::collections::HashSet;

use anyhow::{Context, Result};
use log::{debug, info, warn};
use regex::{Regex, RegexBuilder};
use tracing::instrument;
use url::Url;

use crate::rules::{PROVIDERS, Provider};

mod rules;

/// Result of URL cleaning operation
#[derive(Debug, Clone)]
pub struct CleaningResult {
    /// The cleaned URL string
    pub url: String,
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
            skip_localhost: true,                  // Ignore the local domains
            apply_referral_marketing: true,        // Strip referral marketing
            domain_blocking: true,                 // Block certain domains
            additional_blocked_params: Vec::new(), // Empty extra params
            blacklisted_domains: Vec::new(),       // Empty blacklist
        }
    }
}

/// Main URL cleaner that applies rules to sanitize URLs
#[derive(Debug)]
pub struct UrlCleaner<'a> {
    providers: &'a [Provider],
    options: CleaningOptions,
}

impl<'a> UrlCleaner<'a> {
    /// Create a new URL cleaner from configuration
    pub fn new(options: CleaningOptions) -> Result<Self> {
        info!(
            "Initializing URL cleaner with {} providers",
            PROVIDERS.len()
        );

        Ok(Self {
            providers: PROVIDERS,
            options,
        })
    }

    /// Clean a URL by removing tracking parameters
    #[instrument]
    pub fn clean_url(&self, url: &str) -> Result<CleaningResult> {
        // We need to make this owned for the base manipulation
        let mut url = url.to_string();

        // Add the boilerplate if it's not present
        if !url.starts_with("https://") && !url.starts_with("http://") {
            url = format!("https://{}", url);
        }

        let mut url = Url::parse(&url).context("Failed to parse URL")?;

        debug!("Cleaning URL: {}", url);

        // Check if we should skip this URL
        if self.should_skip_url(&url) {
            debug!("Skipping URL due to configuration: {}", url);
            return Ok(CleaningResult {
                url: url.to_string(),
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
        for provider in self.providers {
            if provider.matches_url(&url)? && !provider.matches_exception(&url)? {
                // Push the matched provider when found
                applied_rules.push(provider.name.to_string());

                let result = self.apply_provider_rules(provider, &mut url)?;

                // Redirect means we're not responsible
                if result.redirect {
                    info!(
                        "URL {} redirected by provider {}",
                        original_url, provider.name
                    );
                    return Ok(CleaningResult {
                        url: url.to_string(),
                        changed: true,
                        redirect: true,
                        cancel: false,
                        applied_rules,
                    });
                }

                // Cancel means we don't need to worry
                if result.cancel {
                    info!("URL {} blocked by provider {}", original_url, provider.name);
                    return Ok(CleaningResult {
                        url: url.to_string(),
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

        // Debug logging
        if changed {
            info!("Cleaned URL: {} -> {}", original_url, url);
        } else {
            debug!("No changes made to URL: {}", url);
        }

        // If not returned early, we can assume this is all correct.
        Ok(CleaningResult {
            url: url.to_string(),
            changed,
            redirect: false,
            cancel: false,
            applied_rules,
        })
    }

    /// Determine if we should skip a URL
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

    /// Detect if the URL is a common localhost URL
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

    /// Apply the rules of the provider to an input url, brings in helper functions to help
    fn apply_provider_rules(&self, provider: &Provider, url: &mut Url) -> Result<CleaningResult> {
        let mut changed = false;
        let mut applied_rules = Vec::new();

        // Check for cancellation (complete provider blocking)
        if provider.complete_provider && self.options.domain_blocking {
            return Ok(CleaningResult {
                url: url.to_string(),
                changed: false,
                redirect: false,
                cancel: true,
                applied_rules: vec![provider.name.to_string()],
            });
        }

        // Check for redirections
        if let Some(redirect_url) = self.apply_redirections(provider, url)? {
            *url = redirect_url;
            return Ok(CleaningResult {
                url: url.to_string(),
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
            url: url.to_string(),
            changed,
            redirect: false,
            cancel: false,
            applied_rules,
        })
    }

    /// Resolve the redirections
    fn apply_redirections(&self, provider: &Provider, url: &Url) -> Result<Option<Url>> {
        for &redirection in provider.redirections.iter() {
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

    /// Apply the specific parameter rules (the most complex of them)
    fn apply_parameter_rules(&self, provider: &Provider, url: &mut Url) -> Result<bool> {
        let mut changed = false;

        // Collect all rules to apply
        let all_rules: &Vec<&'static Regex> = provider.rules.as_ref();
        // if self.options.apply_referral_marketing {
        //     all_rules.extend(provider.referral_marketing.iter());
        // }

        // Remove matching parameters.
        // We only need the key, because that's what the dataset is based on.
        let params_to_remove: Vec<String> = url
            .query_pairs()
            .filter_map(|(key, _)| {
                for &rule in all_rules.iter() {
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

    /// Apply any additional rules that the input specifies
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

impl Provider {
    fn matches_url(&self, url: &Url) -> Result<bool> {
        Ok(self.url_pattern.is_match(url.as_str()))
    }

    fn matches_exception(&self, url: &Url) -> Result<bool> {
        for exception in self.exceptions.iter() {
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
        let cleaner = UrlCleaner::new(CleaningOptions::default()).unwrap();
        let result = cleaner
            .clean_url("https://google.com/search?q=test&utm_source=newsletter")
            .unwrap();

        assert!(result.changed);
        assert_eq!(result.url.as_str(), "https://google.com/search?q=test");
    }

    #[test]
    fn test_additional_params() {
        let options = CleaningOptions {
            additional_blocked_params: vec!["fbclid".to_string(), "gclid".to_string()],
            ..Default::default()
        };

        let cleaner = UrlCleaner::new(options).unwrap();
        let result = cleaner
            .clean_url("https://example.com/?test=1&fbclid=123&gclid=456")
            .unwrap();

        assert!(result.changed);
        assert_eq!(result.url.as_str(), "https://example.com/?test=1");
    }
}
