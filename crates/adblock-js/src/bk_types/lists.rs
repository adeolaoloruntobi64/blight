use super::content_blocking::AdBlockCbRule;

use adblock::{filters::{cosmetic::CosmeticFilterError, network::NetworkFilterError}, lists::{ExpiresInterval, FilterListMetadata, FilterParseError}, FilterSet};
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdBlockExpiresIntervalType {
    Hours,
    Days
}

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen(getter_with_clone)]
pub struct AdBlockExpiresInterval {
    pub interval_type: AdBlockExpiresIntervalType,
    pub amount: u16
}

impl From<ExpiresInterval> for AdBlockExpiresInterval {
    fn from(value: ExpiresInterval) -> Self {
        match value {
            ExpiresInterval::Hours(h) => AdBlockExpiresInterval {
                interval_type: AdBlockExpiresIntervalType::Hours,
                amount: h
            },
            ExpiresInterval::Days(d) => AdBlockExpiresInterval {
                interval_type: AdBlockExpiresIntervalType::Days,
                amount: d as u16
            },
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct AdBlockFilterListMetadata {
    pub homepage: Option<String>,
    pub title: Option<String>,
    pub expires: Option<AdBlockExpiresInterval>,
    pub redirect: Option<String>,
}

#[derive(Debug, Clone, Copy)]
#[wasm_bindgen]
pub enum AdBlockFilterFormat {
    Standard,
    Hosts,
}

#[derive(Debug, Clone, Copy)]
#[wasm_bindgen]
pub enum AdBlockRuleTypes {
    All,
    NetworkOnly,
    CosmeticOnly,
}

#[derive(Copy, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct AdBlockParseOptions {
    pub format: AdBlockFilterFormat,
    pub rule_types: AdBlockRuleTypes,
    pub permissions: u8,
}

#[wasm_bindgen]
impl AdBlockParseOptions {
    
    #[wasm_bindgen(constructor)]
    pub fn new(format: AdBlockFilterFormat, rule_types: AdBlockRuleTypes, permissions: u8) -> Self {
        Self { format, rule_types, permissions }
    }

    pub fn default() -> Self {
        Self {
            format: AdBlockFilterFormat::Standard,
            rule_types: AdBlockRuleTypes::All,
            permissions: 0,
        }
    }
}


#[derive(Clone)]
#[wasm_bindgen]
pub enum AdBlockFilterParseErrorType {
    Unsupported,
    Empty,

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
pub struct AdBlockFilterParseError {
    pub errtype: AdBlockFilterParseErrorType,
    pub message: String
}

impl From<FilterParseError> for AdBlockFilterParseError {
    fn from(value: FilterParseError) -> Self {
        use AdBlockFilterParseErrorType::*;
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
                AdBlockFilterParseError {
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
                AdBlockFilterParseError {
                    errtype,
                    message: c.to_string()
                }
            },
            FilterParseError::Unsupported => AdBlockFilterParseError {
                errtype: Unsupported,
                message: "Unsupported".into()
            },
            FilterParseError::Empty => AdBlockFilterParseError {
                errtype: Empty,
                message: "Empty".into(),
            },
        }
    }
}

#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct ContentBlockingConversionResult {
    pub content_blocking_rules: Vec<AdBlockCbRule>,
    pub filters_used: Vec<String>,
}

#[wasm_bindgen(inspectable)]
#[derive(Default, Clone)]
pub struct AdBlockFilterSet(#[wasm_bindgen(skip)] pub FilterSet);

#[wasm_bindgen]
impl AdBlockFilterSet {

    #[wasm_bindgen(constructor)]
    pub fn new(debug: bool) -> Self {
        Self(FilterSet::new(debug))
    }

    pub fn clone_inner(&self) -> Self {
        self.clone()
    }

    /// Adds the contents of an entire filter list to this FilterSet. Filters that cannot be parsed successfully are ignored. Returns any discovered metadata about the list of rules added.
    pub fn add_filter_list(&mut self, filter_list: &str, opts: AdBlockParseOptions) -> Result<AdBlockFilterListMetadata, JsValue> {
        let FilterListMetadata {
            homepage,
            title,
            expires,
            redirect
        } = self.0.add_filter_list(filter_list, unsafe { std::mem::transmute(opts) });

        Ok(AdBlockFilterListMetadata {
            homepage,
            title,
            expires: expires.map(|exp| AdBlockExpiresInterval::from(exp)),
            redirect
        })
    }
    
    /// Adds a collection of filter rules to this FilterSet. Filters that cannot be parsed successfully are ignored. Returns any discovered metadata about the list of rules added.
    pub fn add_filters(&mut self, filters: Vec<String>, opts: AdBlockParseOptions) -> Result<AdBlockFilterListMetadata, JsValue> {
        let FilterListMetadata {
            homepage,
            title,
            expires,
            redirect
        } = self.0.add_filters(filters, unsafe { std::mem::transmute(opts) });

        Ok(AdBlockFilterListMetadata {
            homepage,
            title,
            expires: expires.map(|exp| AdBlockExpiresInterval::from(exp)),
            redirect
        })
    }

    /// Adds the string representation of a single filter rule to this FilterSet.
    pub fn add_filter(&mut self, filter: String, opts: AdBlockParseOptions) -> Result<(), AdBlockFilterParseError> {
        let opts = unsafe { std::mem::transmute(opts) };
        self.0.add_filter(&filter, opts).map_err(|err| AdBlockFilterParseError::from(err))
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