use std::collections::HashMap;

use adblock::resources::Resource;
use bk_types::resources::AdBlockResource;
use js_sys::Object;
#[cfg(target_family = "wasm")]
use talc::wasm::{new_wasm_dynamic_allocator, WasmDynamicTalc};
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen as serdews;
use wildmatch::WildMatch;

pub mod bk_types;
pub mod resource_assembler;

#[cfg(target_family = "wasm")]
#[global_allocator]
static GLOBAL: WasmDynamicTalc = new_wasm_dynamic_allocator();

pub type JsResult = Result<JsValue, JsError>;

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Since wasm web doesn't have a fs, we have to load the web_accessible_resource_dir somehow.
/// This struct allows for a mapping of the names to the bytes of the file
pub struct AdBlockInlineWebAcessibleResources(HashMap<String, Vec<u8>>);

#[wasm_bindgen]
impl AdBlockInlineWebAcessibleResources {

    #[wasm_bindgen(constructor)]
    // Amazingly, serdews can't convert a js map to a rust hashmap, but can do the reverse
    pub fn new(web_accessible_resources: &JsValue) -> AdBlockInlineWebAcessibleResources {
        AdBlockInlineWebAcessibleResources(serdews::from_value(web_accessible_resources.into()).unwrap())
    }

    pub fn insert(&mut self, name: String, bytes: Vec<u8>) {
        self.0.insert(name, bytes);
    }

    #[wasm_bindgen(js_name = cloneInner)]
    pub fn clone_inner(&self) -> Self {
        self.clone()
    }

    // So instead of a map, we return an Object, because we took in an Object
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Object {
        self.0.serialize(&serdews::Serializer::new().serialize_maps_as_objects(true)).unwrap().into()
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
/// Used to build UBO resources. Uses web_accessible_resources and an optional scriplet.js.
pub struct AdBlockResourceAssemblerInfo(String, AdBlockInlineWebAcessibleResources, Option<String>);

#[wasm_bindgen]
impl AdBlockResourceAssemblerInfo {

    #[wasm_bindgen(constructor)]
    pub fn new(
        redirect_resources_content: String,
        web_accessible_resources: AdBlockInlineWebAcessibleResources,
        scriplet_js_content: Option<String>
    ) -> AdBlockResourceAssemblerInfo {
        AdBlockResourceAssemblerInfo(redirect_resources_content, web_accessible_resources, scriplet_js_content)
    }

    #[wasm_bindgen(js_name = cloneInner)]
    pub fn clone_inner(&self) -> Self {
        self.clone()
    }

    pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
        rmp_serde::to_vec(self).map_err(|e| JsError::new(&format!("There was an error serializing: {e:?}")))
    }

    pub fn deserialize(&mut self, serialized: Vec<u8>) -> Result<(), JsError> {
        *self = rmp_serde::from_slice(&serialized).map_err(|e| JsError::new(&format!("There was an error deserializing: {e:?}")))?;
        Ok(())
    }

}

#[wasm_bindgen]
#[derive(Clone)]
pub struct AdBlockAssembledResources(Vec<Resource>);

#[wasm_bindgen]
impl AdBlockAssembledResources {

    #[wasm_bindgen(constructor)]
    pub fn new(resources: Vec<AdBlockResource>) -> Self {
        Self( unsafe { std::mem::transmute(resources) })
    }

    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Vec<AdBlockResource> {
        unsafe { std::mem::transmute(self.0.clone() ) }
    }

    #[wasm_bindgen(js_name = cloneInner)]
    pub fn clone_inner(&self) -> Self {
        self.clone()
    }

    pub fn serialize_raw(&self) -> Result<Vec<u8>, JsError> {
        rmp_serde::to_vec(&self.0).map_err(|e| JsError::new(&format!("There was an error serializing: {e:?}")))
    }

    pub fn deserialize(&mut self, serialized: &[u8]) -> Result<(), JsError> {
        self.0 = rmp_serde::from_slice(serialized).map_err(|e| JsError::new(&format!("There was an error deserializing: {e:?}")))?;
        Ok(())
    }
}

fn read_resource_from_web_accessible_dir(
    inline_webad: &AdBlockInlineWebAcessibleResources,
    resource_info: &resource_assembler::ResourceProperties,
) -> Option<Resource> {
    let resource_contents = inline_webad.0.get(&resource_info.name)?;
    Some(resource_assembler::build_resource_from_file_contents(resource_contents, resource_info))
}

#[wasm_bindgen]
pub fn assemble_resources(
    resource_assembler_info: &AdBlockResourceAssemblerInfo
) -> Result<AdBlockAssembledResources, JsError> {
    let resource_properties = resource_assembler::read_redirectable_resource_mapping(&resource_assembler_info.0);

    let web_accessible_resources = resource_properties
        .iter()
        .map(|resource_info| {
            read_resource_from_web_accessible_dir(&resource_assembler_info.1, resource_info)
                .ok_or_else(|| JsError::new(&format!("Resource '{}' not found", resource_info.name)))
        })
        .collect::<Result<Vec<_>, JsError>>();

    Ok(AdBlockAssembledResources(web_accessible_resources.map(|mut ress| {
        if let Some(scriptlet_js) = resource_assembler_info.2.as_ref() {
            ress.extend(resource_assembler::read_template_resources(scriptlet_js));
        }
        unsafe { std::mem::transmute(ress) }
    })?))
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct AdBlockExclusionStore(Vec<WildMatch>);

#[wasm_bindgen]
impl AdBlockExclusionStore {

    #[wasm_bindgen(constructor)]
    pub fn new(hosts: Vec<String>) -> Self {
        AdBlockExclusionStore(
            hosts.into_iter()
                .map(|pattern| WildMatch::new(&pattern))
                .collect()
        )
    }

    #[wasm_bindgen(js_name = toArray)]
    pub fn to_array(&self) -> Vec<String> {
        self.0.iter().map(|x| x.to_string()).collect()
    }

    #[wasm_bindgen(js_name = cloneInner)]
    pub fn clone_inner(&self) -> Self {
        self.clone()
    }

    #[wasm_bindgen(js_name = matchHost)]
    pub fn match_host(&self, host: &str) -> Option<String> {
        self.0.iter()
            .find(|pattern| pattern.matches(host))
            .map(|pattern| pattern.to_string())
    }
}