use std::collections::HashSet;

use adblock::{resources::Resource, Engine};
use wasm_bindgen::prelude::*;

use crate::bk_types::{blocker::{VanguardBlockerResult, VanguardUrlSpecificResources}, lists::VanguardFilterSet, request::VanguardRequest, resources::VanguardResource};

#[wasm_bindgen(inspectable)]
pub struct VanguardEngine(Engine);

#[wasm_bindgen]
impl VanguardEngine {
    
    #[wasm_bindgen(constructor)]
    pub fn new(filterset: VanguardFilterSet) -> Self {
        Self(Engine::new_with_filter_set(filterset.0))
    }

    pub fn check_network_request(&self, request: &VanguardRequest) -> VanguardBlockerResult {
        unsafe { std::mem::transmute(self.0.check_network_request(std::mem::transmute(request))) }
    }

    pub fn check_network_request_subset(
        &self,
        request: &VanguardRequest,
        previously_matched_rule: bool,
        force_check_exceptions: bool,
    ) -> VanguardBlockerResult {
        unsafe { std::mem::transmute(self.0.check_network_request_subset(
            std::mem::transmute(request),
            previously_matched_rule,
            force_check_exceptions
        )) }
    }

    pub fn get_csp_directives(&self, request: &VanguardRequest) -> Option<String> {
        unsafe { std::mem::transmute(self.0.get_csp_directives(std::mem::transmute(request))) }
    }

    pub fn hidden_class_id_selectors(&self, classes: Vec<String>, ids: Vec<String>, exceptions: Vec<String>) -> Result<Vec<String>, JsError> {
        let exceptions = HashSet::from_iter(exceptions.into_iter());
        Ok(self.0.hidden_class_id_selectors(&classes, &ids, &exceptions))
    }
    
    pub fn url_cosmetic_resources(&self, url: String) -> VanguardUrlSpecificResources {
        unsafe { std::mem::transmute(self.0.url_cosmetic_resources(&url)) }
    }
    
    pub fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }
    
    pub fn deserialize(&mut self, serialized: &[u8]) -> Result<bool, JsError> {
        self.0.deserialize(serialized).map(|_| true).map_err(|e| JsError::new(&format!("There was an error deserializing: {e:?}")))
    }
    
    pub fn use_resources(&mut self, resources: Vec<VanguardResource>) {
        self.0.use_resources(unsafe { std::mem::transmute::<_, Vec<Resource>>(resources) })
    }
}