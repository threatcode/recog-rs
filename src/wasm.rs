use crate::loader::load_fingerprints_from_xml;
use crate::Matcher;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmRecog {
    matcher: Matcher,
}

#[wasm_bindgen]
pub struct MatchResultJs {
    description: String,
    params: String, // JSON encoded params
}

#[wasm_bindgen]
impl MatchResultJs {
    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn params(&self) -> String {
        self.params.clone()
    }
}

#[wasm_bindgen]
impl WasmRecog {
    #[wasm_bindgen(constructor)]
    pub fn new(xml_content: &str) -> Result<WasmRecog, JsValue> {
        let db = load_fingerprints_from_xml(xml_content)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let matcher = Matcher::new(db);
        Ok(WasmRecog { matcher })
    }

    pub fn match_text(&self, text: &str) -> Result<Vec<MatchResultJs>, JsValue> {
        let results = self.matcher.match_text(text);

        // Convert to JS-friendly format
        let js_results = results
            .into_iter()
            .map(|r| MatchResultJs {
                description: r.fingerprint.description.clone(),
                params: serde_json::to_string(&r.params).unwrap_or_default(),
            })
            .collect();

        Ok(js_results)
    }
}
