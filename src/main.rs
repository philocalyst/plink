use anyhow::Result;
use env_logger;
use log::info;
use plink::{CleaningOptions, UrlCleaner};

fn main() -> Result<()> {
    env_logger::init();

    // Load configuration (you'd typically load this from a file)
    let config_json = include_str!("data.json");

    // Configure options
    let options = CleaningOptions {
        additional_blocked_params: vec![
            "utm_source".to_string(),
            "utm_medium".to_string(),
            "utm_campaign".to_string(),
            "fbclid".to_string(),
            "gclid".to_string(),
        ],
        blacklisted_domains: vec!["trusted-site.com".to_string()],
        ..Default::default()
    };

    // Create cleaner
    let cleaner = UrlCleaner::from_json(config_json, options)?;

    // Clean some URLs
    let urls = vec![
        "https://google.com/search?q=rust&utm_source=newsletter&utm_campaign=weekly",
        "https://www.anglepoise.com/usa/product/type-75-desk-lamp-paul-smith-edition-2/?utm_term=&utm_campaign=Outlet+%7C+Shopping+%7C+US+%7C+Superrb&utm_source=adwords&utm_medium=ppc&hsa_acc=8384491024&hsa_cam=21872127519&hsa_grp=176191620943&hsa_ad=743720003206&hsa_src=g&hsa_tgt=pla-297490888025&hsa_kw=&hsa_mt=&hsa_net=adwords&hsa_ver=3&gad_source=1&gad_campaignid=21872127519&gbraid=0AAAAADuKdZNlL54l3KzzZgZdECJ7uBv_x&gclid=CjwKCAjw6NrBBhB6EiwAvnT_rgkCnerJW6TdzZqUjJ2RsXkr8vKbsW_N1MRK07tQTW_AqQLcfgQurhoCW3sQAvD_BwE",
        "https://facebook.com/page?fbclid=abc123&ref=share",
        "https://example.com/?utm_medium=email&important_param=keep_this",
    ];

    for url in urls {
        let result = cleaner.clean_url(url)?;
        if result.changed {
            println!("Cleaned: {} -> {}", url, result.url);
            println!("Applied rules: {:?}", result.applied_rules);
        } else {
            info!("No changes: {}", url);
        }
    }

    Ok(())
}
