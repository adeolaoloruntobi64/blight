use super::content_blocking::VanguardCbRule;

use adblock::{filters::{cosmetic::CosmeticFilterError, network::NetworkFilterError}, lists::{ExpiresInterval, FilterListMetadata, FilterParseError}, FilterSet};
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VanguardExpiresIntervalType {
    Hours,
    Days
}

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct VanguardExpiresInterval {
    pub interval_type: VanguardExpiresIntervalType,
    pub amount: u16
}

impl From<ExpiresInterval> for VanguardExpiresInterval {
    fn from(value: ExpiresInterval) -> Self {
        match value {
            ExpiresInterval::Hours(h) => VanguardExpiresInterval {
                interval_type: VanguardExpiresIntervalType::Hours,
                amount: h
            },
            ExpiresInterval::Days(d) => VanguardExpiresInterval {
                interval_type: VanguardExpiresIntervalType::Days,
                amount: d as u16
            },
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct VanguardFilterListMetadata {
    pub source_index: usize,
    pub homepage: Option<String>,
    pub title: Option<String>,
    pub expires: Option<VanguardExpiresInterval>,
    pub redirect: Option<String>,
}

#[derive(Debug, Clone, Copy)]
#[wasm_bindgen]
pub enum VanguardFilterFormat {
    Standard,
    Hosts,
}

#[derive(Debug, Clone, Copy)]
#[wasm_bindgen]
pub enum VanguardRuleTypes {
    All,
    NetworkOnly,
    CosmeticOnly,
}

#[derive(Copy, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct VanguardParseOptions {
    pub format: VanguardFilterFormat,
    pub rule_types: VanguardRuleTypes,
    pub permissions: u8,
}

#[wasm_bindgen]
impl VanguardParseOptions {
    
    #[wasm_bindgen(constructor)]
    pub fn new(format: VanguardFilterFormat, rule_types: VanguardRuleTypes, permissions: u8) -> Self {
        Self { format, rule_types, permissions }
    }

    pub fn default() -> Self {
        Self {
            format: VanguardFilterFormat::Standard,
            rule_types: VanguardRuleTypes::All,
            permissions: 0,
        }
    }
}


#[derive(Clone)]
#[wasm_bindgen]
pub enum VanguardFilterParseErrorType {
    Unsupported,
    Empty,
    InvalidExpiresInterval,

    NetworkFilterParseError,
    NetworkNegatedBadFilter,
    NetworkNegatedImportant,
    NetworkNegatedOptionMatchCase,
    NetworkNegatedExplicitCancel,
    NetworkNegatedRedirection,
    NetworkNegatedTag,
    NetworkNegatedGenericHide,
    NetworkNegatedDocument,
    NetworkNegatedAll,
    NetworkGenericHideWithoutException,
    NetworkEmptyRedirection,
    NetworkEmptyRemoveparam,
    NetworkNegatedRemoveparam,
    NetworkRemoveparamWithException,
    NetworkRemoveparamRegexUnsupported,
    NetworkRedirectionUrlInvalid,
    NetworkMultipleModifierOptions,
    NetworkUnrecognisedOption,
    NetworkNoRegex,
    NetworkFullRegexUnsupported,
    NetworkRegexParsingError,
    NetworkPunycodeError,
    NetworkCspWithContentType,
    NetworkMatchCaseWithoutFullRegex,
    NetworkNoSupportedDomains,

    // Cosmetic
    CosmeticPunycodeError,
    CosmeticInvalidActionSpecifier,
    CosmeticUnsupportedSyntax,
    CosmeticMissingSharp,
    CosmeticInvalidCssStyle,
    CosmeticInvalidCssSelector,
    CosmeticGenericUnhide,
    CosmeticGenericScriptInject,
    CosmeticGenericAction,
    CosmeticDoubleNegation,
    CosmeticEmptyRule,
    CosmeticHtmlFilteringUnsupported,
    CosmeticInvalidScriptletArgs,
    CosmeticLocationModifiersUnsupported,
    ProceduralFilterWithMultipleSelectors,
}

#[wasm_bindgen(getter_with_clone)]
pub struct VanguardFilterParseError {
    pub errtype: VanguardFilterParseErrorType,
    pub message: String
}

impl From<FilterParseError> for VanguardFilterParseError {
    fn from(value: FilterParseError) -> Self {
        use VanguardFilterParseErrorType::*;
        match value {
            FilterParseError::Network(n) => {
                let errtype = match n {
                    NetworkFilterError::FilterParseError => NetworkFilterParseError,
                    NetworkFilterError::NegatedBadFilter => NetworkNegatedBadFilter,
                    NetworkFilterError::NegatedImportant => NetworkNegatedImportant,
                    NetworkFilterError::NegatedOptionMatchCase => NetworkNegatedOptionMatchCase,
                    NetworkFilterError::NegatedExplicitCancel => NetworkNegatedExplicitCancel,
                    NetworkFilterError::NegatedRedirection => NetworkNegatedRedirection,
                    NetworkFilterError::NegatedTag => NetworkNegatedTag,
                    NetworkFilterError::NegatedGenericHide => NetworkNegatedGenericHide,
                    NetworkFilterError::NegatedDocument => NetworkNegatedDocument,
                    NetworkFilterError::NegatedAll => NetworkNegatedAll,
                    NetworkFilterError::GenericHideWithoutException => NetworkGenericHideWithoutException,
                    NetworkFilterError::EmptyRedirection => NetworkEmptyRedirection,
                    NetworkFilterError::EmptyRemoveparam => NetworkEmptyRemoveparam,
                    NetworkFilterError::NegatedRemoveparam => NetworkNegatedRemoveparam,
                    NetworkFilterError::RemoveparamWithException => NetworkRemoveparamWithException,
                    NetworkFilterError::RemoveparamRegexUnsupported => NetworkRemoveparamRegexUnsupported,
                    NetworkFilterError::RedirectionUrlInvalid => NetworkRedirectionUrlInvalid,
                    NetworkFilterError::MultipleModifierOptions => NetworkMultipleModifierOptions,
                    NetworkFilterError::UnrecognisedOption => NetworkUnrecognisedOption,
                    NetworkFilterError::NoRegex => NetworkNoRegex,
                    NetworkFilterError::FullRegexUnsupported => NetworkFullRegexUnsupported,
                    NetworkFilterError::RegexParsingError(_) => NetworkRegexParsingError,
                    NetworkFilterError::PunycodeError => NetworkPunycodeError,
                    NetworkFilterError::CspWithContentType => NetworkCspWithContentType,
                    NetworkFilterError::MatchCaseWithoutFullRegex => NetworkMatchCaseWithoutFullRegex,
                    NetworkFilterError::NoSupportedDomains => NetworkNoSupportedDomains,
                    _ => todo!(),
                };
                VanguardFilterParseError {
                    errtype,
                    message: n.to_string()
                }
            },
            FilterParseError::Cosmetic(c) => {
                let errtype = match c {
                    CosmeticFilterError::PunycodeError => CosmeticPunycodeError,
                    CosmeticFilterError::InvalidActionSpecifier => CosmeticInvalidActionSpecifier,
                    CosmeticFilterError::UnsupportedSyntax => CosmeticUnsupportedSyntax,
                    CosmeticFilterError::MissingSharp => CosmeticMissingSharp,
                    CosmeticFilterError::InvalidCssStyle => CosmeticInvalidCssStyle,
                    CosmeticFilterError::InvalidCssSelector => CosmeticInvalidCssSelector,
                    CosmeticFilterError::GenericUnhide => CosmeticGenericUnhide,
                    CosmeticFilterError::GenericScriptInject => CosmeticGenericScriptInject,
                    CosmeticFilterError::GenericAction => CosmeticGenericAction,
                    CosmeticFilterError::DoubleNegation => CosmeticDoubleNegation,
                    CosmeticFilterError::EmptyRule => CosmeticEmptyRule,
                    CosmeticFilterError::HtmlFilteringUnsupported => CosmeticHtmlFilteringUnsupported,
                    CosmeticFilterError::InvalidScriptletArgs => CosmeticInvalidScriptletArgs,
                    CosmeticFilterError::LocationModifiersUnsupported => CosmeticLocationModifiersUnsupported,
                    CosmeticFilterError::ProceduralFilterWithMultipleSelectors => ProceduralFilterWithMultipleSelectors,
                };
                VanguardFilterParseError {
                    errtype,
                    message: c.to_string()
                }
            },
            FilterParseError::Unsupported => VanguardFilterParseError {
                errtype: Unsupported,
                message: "Unsupported".into()
            },
            FilterParseError::Empty => VanguardFilterParseError {
                errtype: Empty,
                message: "Empty".into(),
            },
            FilterParseError::InvalidExpiresInterval => VanguardFilterParseError {
                errtype: InvalidExpiresInterval,
                message: "Invalid Exprires Interval".into(),
            },
        }
    }
}

#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct ContentBlockingConversionResult {
    pub content_blocking_rules: Vec<VanguardCbRule>,
    pub filters_used: Vec<String>,
}

#[wasm_bindgen(inspectable)]
#[derive(Default, Clone)]
pub struct VanguardFilterSet(#[wasm_bindgen(skip)] pub FilterSet);

#[wasm_bindgen]
impl VanguardFilterSet {

    #[wasm_bindgen(constructor)]
    pub fn new(debug: bool) -> Self {
        Self(FilterSet::new(debug))
    }

    pub fn clone_inner(&self) -> Self {
        self.clone()
    }

    /// Adds the contents of an entire filter list to this FilterSet. Filters that cannot be parsed successfully are ignored. Returns any discovered metadata about the list of rules added.
    pub fn add_filter_list(&mut self, filter_list: String, opts: VanguardParseOptions) -> Result<VanguardFilterListMetadata, JsValue> {
        let rec = self.0.add_filter_list(filter_list, unsafe { std::mem::transmute(opts) });
        let FilterListMetadata {
            homepage,
            title,
            expires,
            redirect
        } = rec.metadata;

        Ok(VanguardFilterListMetadata {
            source_index: rec.source_index,
            homepage,
            title,
            expires: expires.map(|exp| VanguardExpiresInterval::from(exp)),
            redirect
        })
    }
    
    pub fn into_content_blocking(self) -> Result<ContentBlockingConversionResult, JsValue> {
        match self.0.into_content_blocking() {
            Ok((cb_rules, filters_used)) => {
                let r = ContentBlockingConversionResult {
                    content_blocking_rules: unsafe { std::mem::transmute(cb_rules) },
                    filters_used,
                };
                Ok(r)
            }
            Err(_) => return Err(JsValue::undefined()),
        }
    }
}