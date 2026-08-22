use std::collections::HashSet;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, Default, PartialEq)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardSourceLocation {
    pub source_index: u32,
    pub line_number: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardFilterRuleDebugInfo {
    pub raw_line: Option<String>,
    pub source_location: Option<VanguardSourceLocation>,
}

#[derive(Debug)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardBlockerResult {
    pub filter: Option<VanguardFilterRuleDebugInfo>,
    pub exception: Option<VanguardFilterRuleDebugInfo>,
    pub important: bool,
    pub redirect: Option<String>,
    pub rewritten_url: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardUrlSpecificResources {
    #[wasm_bindgen(skip)]
    pub hide_selectors: HashSet<String>,
    #[wasm_bindgen(skip)]
    pub procedural_actions: HashSet<String>,
    #[wasm_bindgen(skip)]
    pub exceptions: HashSet<String>,
    pub injected_script: String,
    pub generichide: bool,
}

#[wasm_bindgen]
impl VanguardUrlSpecificResources {

    #[wasm_bindgen(getter = hide_selectors)]
    pub fn get_hide_selectors(&self) -> Vec<String> {
        Vec::from_iter(self.hide_selectors.clone())
    }

    #[wasm_bindgen(setter = hide_selectors)]
    pub fn set_hide_selectors(&mut self, hide_selectors: Vec<String>) {
        self.hide_selectors = HashSet::from_iter(hide_selectors);
    }

    #[wasm_bindgen(getter = procedural_actions)]
    pub fn get_procedural_actions(&self) -> Vec<String> {
        Vec::from_iter(self.procedural_actions.clone())
    }

    #[wasm_bindgen(setter = procedural_actions)]
    pub fn set_procedural_actionss(&mut self, procedural_actions: Vec<String>) {
        self.procedural_actions = HashSet::from_iter(procedural_actions);
    }

    #[wasm_bindgen(getter = exceptions)]
    pub fn get_remove_exceptions(&self) -> Vec<String> {
        Vec::from_iter(self.exceptions.clone().into_iter())
    }

    #[wasm_bindgen(setter = exceptions)]
    pub fn set_remove_exceptions(&mut self, exceptions: Vec<String>)  {
        self.exceptions = HashSet::from_iter(exceptions.into_iter());
    }
}