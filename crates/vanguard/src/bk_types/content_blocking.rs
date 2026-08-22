use std::collections::HashSet;

use wasm_bindgen::prelude::wasm_bindgen;


#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen]
pub enum VanguardCbType {
    Block,
    BlockCookies,
    CssDisplayNone,
    IgnorePreviousRules,
    MakeHttps,
}

#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardCbAction {
    pub typ: VanguardCbType,
    pub selector: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen]
pub enum VanguardCbLoadType {
    FirstParty,
    ThirdParty,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub enum VanguardCbResourceType {
    Document,
    Image,
    StyleSheet,
    Script,
    Font,
    Raw,
    SvgDocument,
    Media,
    Popup,
}

/// Corresponds to the `trigger` field of a Safari content blocking rule.
#[derive(Clone, Debug, Default, PartialEq)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardCbTrigger {
    pub url_filter: String,
    pub url_filter_is_case_sensitive: Option<bool>,
    pub if_domain: Option<Vec<String>>,
    pub unless_domain: Option<Vec<String>>,
    #[wasm_bindgen(skip)]
    pub resource_type: Option<HashSet<VanguardCbResourceType>>,
    pub load_type: Vec<VanguardCbLoadType>,
    pub if_top_url: Option<Vec<String>>,
    pub unless_top_url: Option<Vec<String>>,
}

#[wasm_bindgen]
impl VanguardCbTrigger {
    #[wasm_bindgen(getter = resource_type)]
    pub fn get_resource_type(&self) -> Option<Vec<VanguardCbResourceType>> {
        self.resource_type.clone().map(|hs| Vec::from_iter(hs))
    }

    #[wasm_bindgen(setter = resource_type)]
    pub fn set_resource_type(&mut self, resource_type: Option<Vec<VanguardCbResourceType>>) {
        self.resource_type = resource_type.map(|vec| HashSet::from_iter(vec))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardCbRule {
    pub action: VanguardCbAction,
    pub trigger: VanguardCbTrigger,
}
