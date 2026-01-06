use wasm_bindgen::prelude::*;

use crate::{CleaningOptions, CleaningResult, UrlCleaner};

#[wasm_bindgen]
pub fn clean_url(url: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let options: CleaningOptions =
        serde_wasm_bindgen::from_value(options).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let cleaner = UrlCleaner::new(options).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = cleaner
        .clean_url(url)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Convert Url to String
    let result = CleaningResult {
        url: result.url.to_string(),
        changed: result.changed,
        redirect: result.redirect,
        cancel: result.cancel,
        applied_rules: result.applied_rules,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn clean_url_simple(url: &str) -> Result<String, JsValue> {
    let cleaner = UrlCleaner::new(CleaningOptions::default())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = cleaner
        .clean_url(url)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(result.url.to_string())
}

#[wasm_bindgen]
pub fn default_options() -> JsValue {
    serde_wasm_bindgen::to_value(&CleaningOptions::default()).unwrap()
}
