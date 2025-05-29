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
        "https://www.amazon.com/dp/exampleProduct/ref=sxin_0_pb?__mk_de_DE=ÅMÅŽÕÑ&keywords=tea&pd_rd_i=exampleProduct&pd_rd_r=8d39e4cd-1e4f-43db-b6e7-72e969a84aa5&pd_rd_w=1pcKM&pd_rd_wg=hYrNl&pf_rd_p=50bbfd25-5ef7-41a2-68d6-74d854b30e30&pf_rd_r=0GMWD0YYKA7XFGX55ADP&qid=1517757263&rnid=2914120011",
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
