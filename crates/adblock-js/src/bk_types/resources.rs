use adblock::resources::{MimeType, ResourceType};
use base64::{Engine, engine::general_purpose::STANDARD};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub enum AdBlockResourceType {
    MimeTextCss,
    MimeImageGif,
    MimeTextHtml,
    MimeApplicationJavascript,
    MimeApplicationJson,
    MimeAudioMp3,
    MimeVideoMp4,
    MimeImagePng,
    MimeTextPlain,
    MimeTextXml,
    MimeFnJavascript,
    MimeUnknown,
    Template,
}

impl From<ResourceType> for AdBlockResourceType {
    fn from(value: ResourceType) -> Self {
        use AdBlockResourceType::*;
        match value {
            ResourceType::Mime(m) => match m {
                MimeType::TextCss => MimeTextCss,
                MimeType::ImageGif => MimeImageGif,
                MimeType::TextHtml => MimeTextHtml,
                MimeType::ApplicationJavascript => MimeApplicationJavascript,
                MimeType::ApplicationJson => MimeApplicationJson,
                MimeType::AudioMp3 => MimeAudioMp3,
                MimeType::VideoMp4 => MimeVideoMp4,
                MimeType::ImagePng => MimeImagePng,
                MimeType::TextPlain => MimeTextPlain,
                MimeType::TextXml => MimeTextXml,
                MimeType::FnJavascript => MimeFnJavascript,
                MimeType::Unknown => MimeUnknown,
            },
            ResourceType::Template => Template,
        }
    }
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct AdBlockResource {
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: AdBlockResourceType,
    pub content: String,
    pub dependencies: Vec<String>,
    pub permission: u8,
}

#[wasm_bindgen]
impl AdBlockResource {
    #[wasm_bindgen(constructor)]
    pub fn new(
        name: String,
        aliases: Vec<String>,
        kind: AdBlockResourceType,
        content: String,
        dependencies: Vec<String>,
        permission: u8
    ) -> Self {
        Self {
            name,
            aliases,
            kind,
            content: STANDARD.encode(content),
            dependencies,
            permission,
        }
    }
}