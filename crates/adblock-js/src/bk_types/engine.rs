use std::collections::HashSet;

use adblock::{resources::Resource, Engine};
use wasm_bindgen::prelude::*;

use crate::bk_types::{blocker::{AdBlockBlockerResult, AdBlockUrlSpecificResources}, lists::AdBlockFilterSet, request::AdBlockRequest, resources::AdBlockResource};

#[wasm_bindgen(inspectable)]
pub struct AdBlockEngine(Engine);

#[wasm_bindgen]
impl AdBlockEngine {
    
    #[wasm_bindgen(constructor)]
    pub fn new(filterset: AdBlockFilterSet, optimize: Option<bool>) -> Self {
        let optimize = optimize.unwrap_or(true);
        Self(Engine::from_filter_set(filterset.0, optimize))
    }

    pub fn check_network_request(&self, request: AdBlockRequest) -> AdBlockBlockerResult {
        unsafe { std::mem::transmute(self.0.check_network_request(&std::mem::transmute(request))) }
    }

    pub fn hidden_class_id_selectors(&self, classes: Vec<String>, ids: Vec<String>, exceptions: Vec<String>) -> Result<Vec<String>, JsError> {
        let exceptions = HashSet::from_iter(exceptions.into_iter());
        Ok(self.0.hidden_class_id_selectors(&classes, &ids, &exceptions))
    }
    
    pub fn url_cosmetic_resources(&self, url: String) -> AdBlockUrlSpecificResources {
        unsafe { std::mem::transmute(self.0.url_cosmetic_resources(&url)) }
    }
    
    pub fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }
    
    pub fn deserialize(&mut self, serialized: &[u8]) -> Result<bool, JsError> {
        self.0.deserialize(serialized).map(|_| true).map_err(|e| JsError::new(&format!("There was an error deserializing: {e:?}")))
    }
    
    pub fn enable_tags(&mut self, tags: Vec<String>) {
        self.0.enable_tags(&tags.iter().map(|a| a.as_str()).collect::<Vec<&str>>())
    }
    
    pub fn disable_tags(&mut self, tags: Vec<String>) {
        self.0.disable_tags(&tags.iter().map(|a| a.as_str()).collect::<Vec<&str>>())
    }

    pub fn use_tags(&mut self, tags: Vec<String>) {
        self.0.use_tags(&tags.iter().map(|a| a.as_str()).collect::<Vec<&str>>())
    }
        
    pub fn tag_exists(&self, tag: String) -> bool {
        self.0.tag_exists(&tag)
    }
    
    pub fn use_resources(&mut self, resources: Vec<AdBlockResource>) {
        self.0.use_resources(unsafe { std::mem::transmute::<_, Vec<Resource>>(resources) })
    }
}