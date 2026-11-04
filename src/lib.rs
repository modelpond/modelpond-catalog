use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
#[cfg(test)]
use std::path::Path;
use std::sync::OnceLock;

use rust_embed::RustEmbed;
use serde::Deserialize;
use serde::Serialize;

pub const COMPILE_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
pub const USD_TO_EUR_RATE: f64 = 0.87237;

static CATALOG: OnceLock<Catalog> = OnceLock::new();

#[derive(RustEmbed)]
#[folder = "data"]
struct CatalogAssets;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    OpenAi,
    OpenRouter,
    Anthropic,
    #[serde(rename = "mistral")]
    Mistral,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAi => write!(formatter, "openai"),
            Self::OpenRouter => write!(formatter, "openrouter"),
            Self::Anthropic => write!(formatter, "anthropic"),
            Self::Mistral => write!(formatter, "mistral"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub config: Config,
    pub providers: Vec<Provider>,
    pub vendors: Vec<Vendor>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub pricing_policy: PricingPolicy,
    pub sections: Vec<Section>,
    pub icons: BTreeMap<String, String>,
    pub modality_order: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PricingPolicy {
    pub currency: String,
    pub provider_list_prices: bool,
    pub modelpond_markup_percent: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub logo: String,
    pub country: String,
    pub blurb: String,
    pub urls: BTreeMap<String, ProviderLink>,
    pub regions: Vec<ProviderRegion>,
    pub models: Vec<ProviderModel>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderLink {
    pub label: String,
    pub url: String,
}

impl Provider {
    pub fn homepage_url(&self) -> Option<&str> {
        self.urls.get("homepage").map(|link| link.url.as_str())
    }

    pub fn privacy_url(&self) -> Option<&str> {
        self.urls.get("privacy").map(|link| link.url.as_str())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderRegion {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    #[serde(skip_serializing)]
    pub endpoint: Option<String>,
    pub api_convention: Option<ProviderType>,
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderModel {
    pub model_id: String,
    pub vendor_model_id: String,
    pub api_model: String,
    /// Optional provider-imposed limit below the model's native context window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window_cap_tokens: Option<u64>,
    pub pricing: HostedPricing,
    pub serving_regions: Vec<String>,
    pub note: Option<String>,
    pub input_modalities: Option<Vec<String>>,
    pub output_modalities: Option<Vec<String>>,
    pub restricted_effort_settings: Option<EffortSettings>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendModel<'a> {
    pub provider_type: &'a ProviderType,
    pub endpoint: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
}

impl ProviderModel {
    pub fn effort_settings<'a>(&'a self, model: &'a VendorModel) -> &'a EffortSettings {
        self.restricted_effort_settings
            .as_ref()
            .unwrap_or(&model.effort_settings)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostedPricing {
    pub currency: String,
    pub input: Price,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_input: Option<Price>,
    pub output: Price,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Price {
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub dp: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffortSettings {
    pub supported: Vec<String>,
    pub default: String,
}

impl Default for EffortSettings {
    fn default() -> Self {
        Self {
            supported: vec!["none".to_string()],
            default: "none".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Vendor {
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
    #[serde(default)]
    pub homepage_featured: bool,
    pub full_name: Option<String>,
    pub country: String,
    pub url: Option<String>,
    pub blurb: Option<String>,
    pub models: Vec<VendorModel>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VendorModel {
    pub id: String,
    pub name: String,
    /// User-facing model family used to group editions in selectors.
    pub family: String,
    /// User-facing edition label, excluding size and checkpoint metadata.
    pub edition: String,
    /// Size or parameter designation as labelled by the model publisher.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Internal checkpoint identifier, intentionally hidden from end users.
    #[serde(default, skip_serializing)]
    pub checkpoint: Option<String>,
    #[serde(rename = "type")]
    pub model_type: String,
    pub release_date: Option<String>,
    pub aa_score: Option<f64>,
    pub aa_report_url: Option<String>,
    pub blurb: String,
    /// Whether this model belongs in the catalogue's flagship tier.
    #[serde(default)]
    pub flagship: bool,
    /// Canonical downloadable tokenizer definition for the model.
    pub tokenizer_url: Option<String>,
    /// Maximum model-native context window, before provider caps.
    pub context_window_tokens: Option<u64>,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    #[serde(default)]
    pub effort_settings: EffortSettings,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    #[serde(rename = "type")]
    pub model_type: String,
    pub blurb: String,
}

fn validate_effort_settings(
    settings: &EffortSettings,
    label: &str,
    effort_levels: &HashSet<&str>,
) -> Result<(), String> {
    if settings.supported.is_empty() {
        return Err(format!(
            "model '{label}' has no supported reasoning efforts"
        ));
    }

    for effort in &settings.supported {
        if !effort_levels.contains(effort.as_str()) {
            return Err(format!(
                "model '{label}' has unknown reasoning effort '{effort}'"
            ));
        }
    }

    if !settings.supported.contains(&settings.default) {
        return Err(format!(
            "model '{label}' default reasoning effort '{}' is unsupported",
            settings.default
        ));
    }

    Ok(())
}

fn provider_model_key(provider_id: &str, region_id: &str, model_id: &str) -> String {
    format!("{provider_id}/{region_id}/{model_id}")
}

impl Catalog {
    pub fn provider_model(
        &self,
        hosted_model_id: &str,
    ) -> Option<(&Provider, &ProviderRegion, &ProviderModel)> {
        self.providers.iter().find_map(|provider| {
            provider.models.iter().find_map(|model| {
                provider
                    .regions
                    .iter()
                    .find(|region| {
                        provider_model_key(&provider.id, &region.id, &model.model_id)
                            == hosted_model_id
                    })
                    .map(|region| (provider, region, model))
            })
        })
    }

    pub fn backend_model(&self, hosted_model_id: &str) -> Option<BackendModel<'_>> {
        let (_, region, hosted) = self.provider_model(hosted_model_id)?;
        Some(BackendModel {
            provider_type: region.api_convention.as_ref()?,
            endpoint: region.endpoint.as_deref()?,
            api_key: region.api_key.as_deref()?,
            model: &hosted.api_model,
        })
    }

    pub fn vendor_model(&self, model_id: &str) -> Option<(&Vendor, &VendorModel)> {
        self.vendors.iter().find_map(|vendor| {
            vendor
                .models
                .iter()
                .find(|model| model.id == model_id)
                .map(|model| (vendor, model))
        })
    }

    fn validate(&self) -> Result<(), String> {
        let modality_order = self
            .config
            .modality_order
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let effort_levels = HashSet::from(["none", "low", "medium", "high", "max"]);
        let mut provider_ids = HashSet::new();
        let mut hosted_model_ids = HashSet::new();
        for provider in &self.providers {
            if !provider_ids.insert(provider.id.as_str()) {
                return Err(format!("duplicate provider id '{}'", provider.id));
            }
            if provider.regions.is_empty() {
                return Err(format!("provider '{}' has no serving regions", provider.id));
            }
            let logo_path = format!("providers/{}", provider.logo);
            if !data_file_exists(&logo_path) {
                return Err(format!(
                    "provider '{}' logo asset '{logo_path}' is missing",
                    provider.id
                ));
            }
            for region in &provider.regions {
                if region.label.trim().is_empty() {
                    return Err(format!(
                        "region '{}:{}' has no serving location",
                        provider.id, region.id
                    ));
                }
            }

            let region_ids = provider
                .regions
                .iter()
                .map(|region| region.id.as_str())
                .collect::<HashSet<_>>();
            for model in &provider.models {
                let model_label = format!("{}/{}", provider.id, model.model_id);
                let (_, vendor_model) =
                    self.vendor_model(&model.vendor_model_id).ok_or_else(|| {
                        format!(
                            "provider model '{model_label}' references unknown vendor model '{}'",
                            model.vendor_model_id
                        )
                    })?;
                if model.serving_regions.is_empty() {
                    return Err(format!(
                        "provider model '{model_label}' has no serving regions"
                    ));
                }
                if let Some(cap) = model.context_window_cap_tokens {
                    if cap == 0 {
                        return Err(format!(
                            "provider model '{model_label}' context cap must be positive"
                        ));
                    }
                    if let Some(context_window) = vendor_model.context_window_tokens {
                        if cap > context_window {
                            return Err(format!(
                                "provider model '{model_label}' context cap {cap} exceeds vendor context window {context_window}"
                            ));
                        }
                    }
                }
                let input_modalities = model
                    .input_modalities
                    .as_deref()
                    .unwrap_or(&vendor_model.input_modalities);
                let output_modalities = model
                    .output_modalities
                    .as_deref()
                    .unwrap_or(&vendor_model.output_modalities);
                let audio_model = input_modalities
                    .iter()
                    .chain(output_modalities)
                    .any(|modality| modality == "audio");
                if model.pricing.input.value.is_none() {
                    return Err(format!("provider model '{model_label}' has no input price"));
                }
                if !audio_model && model.pricing.output.value.is_none() {
                    return Err(format!(
                        "provider model '{model_label}' has no output price"
                    ));
                }
                for modality in model
                    .input_modalities
                    .iter()
                    .flatten()
                    .chain(model.output_modalities.iter().flatten())
                {
                    if !modality_order.contains(modality.as_str()) {
                        return Err(format!(
                            "provider model '{model_label}' has unknown modality '{modality}'"
                        ));
                    }
                }
                for region_id in &model.serving_regions {
                    if !region_ids.contains(region_id.as_str()) {
                        return Err(format!(
                            "provider model '{model_label}' references unknown region '{region_id}'"
                        ));
                    }
                    let key = provider_model_key(&provider.id, region_id, &model.model_id);
                    if !hosted_model_ids.insert(key.clone()) {
                        return Err(format!("duplicate provider model key '{key}'"));
                    }
                }
                if let Some(settings) = &model.restricted_effort_settings {
                    validate_effort_settings(settings, &model_label, &effort_levels)?;
                }
            }
        }

        for vendor in &self.vendors {
            for model in &vendor.models {
                validate_effort_settings(&model.effort_settings, &model.id, &effort_levels)?;
            }
        }

        let mut section_types = HashSet::new();
        for section in &self.config.sections {
            if !section_types.insert(section.model_type.as_str()) {
                return Err(format!("duplicate model type '{}'", section.model_type));
            }
        }

        let mut vendor_ids = HashSet::new();
        let mut model_ids = HashSet::new();
        let hosted_vendor_model_ids = self
            .providers
            .iter()
            .flat_map(|provider| &provider.models)
            .map(|model| model.vendor_model_id.as_str())
            .collect::<HashSet<_>>();
        for vendor in &self.vendors {
            if !vendor_ids.insert(vendor.id.as_str()) {
                return Err(format!("duplicate vendor id '{}'", vendor.id));
            }
            if let Some(logo) = &vendor.logo {
                let logo_path = format!("models/{logo}");
                if !data_file_exists(&logo_path) {
                    return Err(format!(
                        "vendor '{}' logo asset '{logo_path}' is missing",
                        vendor.id
                    ));
                }
            }
            for model in &vendor.models {
                if !hosted_vendor_model_ids.contains(model.id.as_str()) {
                    return Err(format!("vendor model '{}' has no provider entry", model.id));
                }
                if model.family.trim().is_empty() {
                    return Err(format!("model '{}' has no family", model.id));
                }
                if model.edition.trim().is_empty() {
                    return Err(format!("model '{}' has no edition", model.id));
                }
                if model
                    .size
                    .as_deref()
                    .is_some_and(|size| size.trim().is_empty())
                {
                    return Err(format!("model '{}' has an empty size", model.id));
                }
                if model
                    .checkpoint
                    .as_deref()
                    .is_some_and(|checkpoint| checkpoint.trim().is_empty())
                {
                    return Err(format!("model '{}' has an empty checkpoint", model.id));
                }
                if !model_ids.insert(model.id.as_str()) {
                    return Err(format!("duplicate model id '{}'", model.id));
                }
                if model.context_window_tokens == Some(0) {
                    return Err(format!(
                        "model '{}' context window must be positive",
                        model.id
                    ));
                }
                if model
                    .tokenizer_url
                    .as_deref()
                    .is_some_and(|url| !url.starts_with("https://"))
                {
                    return Err(format!("model '{}' tokenizer URL must use HTTPS", model.id));
                }
                if !section_types.contains(model.model_type.as_str()) {
                    return Err(format!(
                        "model '{}' has unknown type '{}'",
                        model.id, model.model_type
                    ));
                }
                for modality in model
                    .input_modalities
                    .iter()
                    .chain(&model.output_modalities)
                {
                    if !modality_order.contains(modality.as_str()) {
                        return Err(format!(
                            "model '{}' has unknown modality '{}'",
                            model.id, modality
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

fn read_data_file(relative: &str) -> anyhow::Result<String> {
    let source = CatalogAssets::get(relative)
        .ok_or_else(|| anyhow::anyhow!("catalog asset '{relative}' is missing"))?;
    String::from_utf8(source.data.into_owned())
        .map_err(|error| anyhow::anyhow!("catalog asset '{relative}' is not UTF-8: {error}"))
}

fn data_files(directory: &str) -> anyhow::Result<Vec<String>> {
    let prefix = format!("{directory}/");
    let mut names = CatalogAssets::iter()
        .filter(|name| name.starts_with(&prefix) && name.ends_with(".yaml"))
        .map(Cow::into_owned)
        .collect::<Vec<_>>();
    names.sort();
    Ok(names)
}

fn data_file_exists(relative: &str) -> bool {
    CatalogAssets::get(relative).is_some()
}

pub fn asset(relative: &str) -> Option<Cow<'static, [u8]>> {
    CatalogAssets::get(relative).map(|asset| asset.data)
}

pub fn load() -> anyhow::Result<Catalog> {
    let config_source = read_data_file("config.yaml")?;
    let config: Config = serde_yaml::from_str(&config_source)
        .map_err(|error| anyhow::anyhow!("config.yaml is invalid: {error}"))?;
    let providers = data_files("providers")?
        .into_iter()
        .map(|filename| {
            let source = read_data_file(&filename)?;
            let provider: Provider = serde_yaml::from_str(&source)
                .map_err(|error| anyhow::anyhow!("{filename} is invalid: {error}"))?;
            let expected_id = filename
                .strip_prefix("providers/")
                .and_then(|name| name.strip_suffix(".yaml"))
                .ok_or_else(|| anyhow::anyhow!("invalid provider filename '{filename}'"))?;
            if provider.id != expected_id {
                anyhow::bail!("provider file '{filename}' must contain provider '{expected_id}'");
            }
            Ok(provider)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let vendors = data_files("models")?
        .into_iter()
        .map(|filename| {
            let source = read_data_file(&filename)?;
            let vendor: Vendor = serde_yaml::from_str(&source)
                .map_err(|error| anyhow::anyhow!("{filename} is invalid: {error}"))?;
            let expected_id = filename
                .strip_prefix("models/")
                .and_then(|name| name.strip_suffix(".yaml"))
                .ok_or_else(|| anyhow::anyhow!("invalid model filename '{filename}'"))?;
            if vendor.id != expected_id {
                anyhow::bail!("model file '{filename}' must contain vendor '{expected_id}'");
            }
            Ok(vendor)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let catalog = Catalog {
        config,
        providers,
        vendors,
    };
    catalog.validate().map_err(anyhow::Error::msg)?;
    Ok(catalog)
}

pub fn data() -> &'static Catalog {
    CATALOG.get_or_init(|| load().unwrap_or_else(|error| panic!("invalid model catalog: {error}")))
}

pub fn convert_currency(value: f64, from: &str, to: &str, decimals: u32) -> Option<f64> {
    if !value.is_finite() {
        return None;
    }
    if from == to {
        return Some(value);
    }
    let converted = match (from, to) {
        ("USD", "EUR") => value * USD_TO_EUR_RATE,
        ("EUR", "USD") => value / USD_TO_EUR_RATE,
        _ => return None,
    };
    let precision = 10_f64.powi(i32::try_from(decimals).ok()?);
    if !precision.is_finite() {
        return None;
    }
    Some((converted * precision).ceil() / precision)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_conversion_rounds_up_to_display_precision() {
        assert_eq!(convert_currency(1.4, "USD", "EUR", 2), Some(1.23));
        assert_eq!(convert_currency(1.4, "USD", "EUR", 3), Some(1.222));
        assert_eq!(convert_currency(1.4, "EUR", "EUR", 2), Some(1.4));
        assert_eq!(convert_currency(1.4, "GBP", "EUR", 2), None);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_loader_reads_the_compile_time_data_directory() {
        let path = Path::new(COMPILE_DATA_DIR).join("config.yaml");
        assert_eq!(
            read_data_file("config.yaml").expect("debug asset should load"),
            std::fs::read_to_string(path).expect("compile-time data should be readable")
        );
    }

    #[test]
    fn catalog_yaml_deserializes_separate_provider_and_model_data() {
        let catalog = data();
        assert_eq!(catalog.config.pricing_policy.currency, "EUR");
        assert_eq!(catalog.config.sections.len(), 7);
        assert_eq!(catalog.providers.len(), 14);
        assert_eq!(
            catalog
                .providers
                .iter()
                .map(|provider| provider.models.len())
                .sum::<usize>(),
            164
        );
        assert_eq!(catalog.vendors.len(), 24);
        assert_eq!(
            catalog
                .vendors
                .iter()
                .map(|vendor| vendor.models.len())
                .sum::<usize>(),
            89
        );
    }

    #[test]
    fn provider_pricing_and_vendor_model_metadata_are_separate() {
        let catalog = data();
        let (provider, region, hosted) = catalog
            .provider_model("scaleway/fr-par/qwen3.8-27b")
            .expect("hosted model should exist");
        let (vendor, model) = catalog
            .vendor_model(&hosted.vendor_model_id)
            .expect("vendor model should exist");

        assert_eq!(provider.id, "scaleway");
        assert_eq!(region.id, "fr-par");
        assert_eq!(vendor.name, "Alibaba");
        assert_eq!(model.name, "Qwen3.8 27B");
        assert_eq!(model.family, "Qwen");
        assert_eq!(model.edition, "3.8");
        assert_eq!(model.size.as_deref(), Some("27B"));
        assert_eq!(model.checkpoint, None);
        assert_eq!(model.model_type, "Text");
        assert_eq!(hosted.pricing.currency, "EUR");
        assert_eq!(hosted.pricing.output.value, Some(3.3));
        assert_eq!(model.aa_score, Some(33.7));
    }

    #[test]
    fn model_families_preserve_edition_order() {
        let catalog = data();
        let family_ids = |family: &str| {
            catalog
                .vendors
                .iter()
                .flat_map(|vendor| &vendor.models)
                .filter(|model| model.family == family)
                .map(|model| model.id.as_str())
                .collect::<Vec<_>>()
        };

        assert_eq!(
            &family_ids("GLM")[..3],
            ["glm-5.3", "glm-5.3-flash", "glm-5.3-flash-system-one"]
        );
        assert_eq!(
            family_ids("DeepSeek"),
            [
                "deepseek-v4-flash-0731",
                "deepseek-v4-pro-0813",
                "deepseek-v4-pro",
                "deepseek-v4.1-flash",
                "deepseek-v4.1-flash-1017",
                "deepseek-ocr-2",
            ]
        );
        let (_, flash) = catalog
            .vendor_model("deepseek-v4-flash-0731")
            .expect("DeepSeek Flash should exist");
        assert_eq!(flash.edition, "4 Flash");
        assert_eq!(flash.size, None);
        assert_eq!(flash.checkpoint.as_deref(), Some("0731"));
        let serialized = serde_yaml::to_string(flash).expect("model should serialize");
        assert!(serialized.contains("family: DeepSeek"));
        assert!(serialized.contains("edition: 4 Flash"));
        assert!(!serialized.contains("checkpoint:"));

        let (_, glm) = catalog
            .vendor_model("glm-5.3")
            .expect("GLM 5.3 should exist");
        assert_eq!(glm.edition, "5.3");
        assert_eq!(glm.size, None);
        assert_eq!(glm.checkpoint, None);
    }

    #[test]
    fn scaleway_priced_models_record_context_and_tokenizers() {
        let catalog = data();
        for (model_id, context_window) in [
            ("glm-5.3", 1_048_576),
            ("glm-5.2", 1_048_576),
            ("deepseek-v4.1-flash-1017", 1_048_576),
            ("deepseek-v4-flash-0731", 1_048_576),
            ("qwen3.8-27b", 262_144),
            ("qwen3.6-35b-a3b", 262_144),
            ("qwen3.5-397b-a17b", 262_144),
            ("qwen3-235b-a22b-instruct-2507", 262_144),
            ("qwen3-embedding-8b", 40_960),
            ("gemma-4-26b-a4b-it", 262_144),
            ("gpt-oss-120b", 131_072),
            ("whisper-large-v3", 0),
            ("lightonocr-3-0.8b", 262_144),
            ("llama-3.3-70b-instruct", 131_072),
            ("mistral-medium-3.5-128b", 1_048_576),
            ("mistral-small-3.2-24b-instruct-2506", 131_072),
            ("bge-multilingual-gemma2", 8_192),
        ] {
            let (_, model) = catalog.vendor_model(model_id).expect("model should exist");
            assert!(
                model
                    .tokenizer_url
                    .as_deref()
                    .is_some_and(|url| url.starts_with("https://huggingface.co/")),
                "model '{}' is missing its tokenizer URL",
                model.id
            );
            if context_window == 0 {
                assert_eq!(model.context_window_tokens, None);
            } else {
                assert_eq!(
                    model.context_window_tokens,
                    Some(context_window),
                    "model '{}' has the wrong context window",
                    model.id
                );
            }
        }
    }

    #[test]
    fn scaleway_models_record_provider_context_caps() {
        let catalog = data();
        for (model_id, context_cap) in [
            ("deepseek-v4-flash-0731", 256_000),
            ("deepseek-v4.1-flash-1017", 256_000),
            ("glm-5.3", 256_000),
            ("glm-5.2", 256_000),
            ("qwen3.8-27b", 256_000),
            ("qwen3.6-35b-a3b", 256_000),
            ("gemma-4-26b-a4b-it", 256_000),
            ("mistral-medium-3.5-128b", 256_000),
            ("llama-3.3-70b-instruct", 128_000),
            ("qwen3-235b-a22b-instruct-2507", 250_000),
            ("qwen3.5-397b-a17b", 250_000),
            ("mistral-small-3.2-24b-instruct-2506", 128_000),
            ("gpt-oss-120b", 128_000),
        ] {
            let (_, _, hosted) = catalog
                .provider_model(&format!("scaleway/fr-par/{model_id}"))
                .expect("Scaleway model should exist");
            assert_eq!(hosted.context_window_cap_tokens, Some(context_cap));
        }
    }

    #[test]
    fn all_provider_models_have_required_prices() {
        let catalog = data();
        for provider in &catalog.providers {
            for model in &provider.models {
                let (_, vendor_model) = catalog
                    .vendor_model(&model.vendor_model_id)
                    .expect("provider model should reference vendor model");
                let input = model
                    .input_modalities
                    .as_deref()
                    .unwrap_or(&vendor_model.input_modalities);
                let output = model
                    .output_modalities
                    .as_deref()
                    .unwrap_or(&vendor_model.output_modalities);
                let audio_model = input
                    .iter()
                    .chain(output)
                    .any(|modality| modality == "audio");
                assert!(
                    model.pricing.input.value.is_some(),
                    "provider model '{}/{}' has no input price",
                    provider.id,
                    model.model_id
                );
                if !audio_model {
                    assert!(
                        model.pricing.output.value.is_some(),
                        "provider model '{}/{}' has no output price",
                        provider.id,
                        model.model_id
                    );
                }
            }
        }
    }

    #[test]
    fn backend_model_resolves_call_configuration() {
        let backend = data()
            .backend_model("scaleway/fr-par/mistral-small-3.2-24b-instruct-2506")
            .expect("backend model should resolve");
        assert_eq!(backend.provider_type, &ProviderType::OpenAi);
        assert_eq!(backend.endpoint, "https://api.scaleway.ai/v1");
        assert_eq!(backend.api_key, "SCALEWAY_API_KEY");
        assert_eq!(backend.model, "mistral-small-3.2-24b-instruct-2506");
        assert!(data().backend_model("missing/model").is_none());
    }

    #[test]
    fn pricing_keeps_the_provider_currency() {
        let catalog = data();
        let (_, _, ark) = catalog
            .provider_model("ark/pl/qwen3.5-122b-a10b")
            .expect("ARK model should exist");
        let (_, _, mistral) = catalog
            .provider_model("mistral/fr/mistral-medium-3-5")
            .expect("Mistral model should exist");
        let (_, _, scaleway) = catalog
            .provider_model("scaleway/fr-par/qwen3.8-27b")
            .expect("Scaleway model should exist");

        assert_eq!(ark.pricing.currency, "USD");
        assert_eq!(mistral.pricing.currency, "USD");
        assert_eq!(scaleway.pricing.currency, "EUR");
    }

    #[test]
    fn system_one_models_are_model_types() {
        let system_one_models = data()
            .vendors
            .iter()
            .flat_map(|vendor| &vendor.models)
            .filter(|model| model.model_type == "System One")
            .map(|model| model.id.as_str())
            .collect::<HashSet<_>>();

        for model_id in [
            "clef-flash",
            "glm-5.3-flash-system-one",
            "laya-english",
            "laya-multilingual",
            "laya-typed-decisions",
            "strom-1.0.7",
        ] {
            assert!(
                system_one_models.contains(model_id),
                "missing System One model '{model_id}'"
            );
        }
    }

    #[test]
    fn structured_outputs_have_separate_modalities() {
        let catalog = data();
        for (model_id, modality) in [
            ("qwen3-embedding-8b", "embedding"),
            ("bge-reranker-v2-m3", "reranker"),
            ("laya-typed-decisions", "typed_decision"),
        ] {
            let (_, model) = catalog
                .vendor_model(model_id)
                .expect("structured output model should exist");
            assert_eq!(model.output_modalities, vec![modality.to_string()]);
        }
    }

    #[test]
    fn modalities_are_flat_provider_and_model_fields() {
        let catalog = data();
        let (_, _, kimi) = catalog
            .provider_model("berget/se/kimi-k3")
            .expect("Kimi should be hosted");
        let (_, model) = catalog
            .vendor_model(&kimi.vendor_model_id)
            .expect("Kimi model should exist");

        assert_eq!(kimi.input_modalities, Some(vec!["text".to_string()]));
        assert!(kimi.output_modalities.is_none());
        assert_eq!(model.input_modalities, vec!["image", "text"]);
        assert_eq!(model.output_modalities, vec!["text"]);
    }

    #[test]
    fn provider_urls_use_fixed_names() {
        let catalog = data();
        let scaleway = catalog
            .providers
            .iter()
            .find(|provider| provider.id == "scaleway")
            .expect("Scaleway should exist");
        let mistral = catalog
            .providers
            .iter()
            .find(|provider| provider.id == "mistral")
            .expect("Mistral should exist");

        assert_eq!(
            scaleway.privacy_url(),
            Some("https://www.scaleway.com/en/privacy-policy/")
        );
        assert_eq!(scaleway.country, "France");
        assert!(scaleway.blurb.contains("Paris"));
        assert_eq!(scaleway.homepage_url(), Some("https://www.scaleway.com/"));
        assert!(scaleway.urls.contains_key("contracts"));
        assert!(mistral.urls.contains_key("dpa"));
        assert!(mistral.urls.contains_key("zdr"));
        assert!(
            catalog
                .providers
                .iter()
                .all(|provider| !provider.blurb.is_empty())
        );
    }

    #[test]
    fn provider_logos_exist_beside_their_yaml_files() {
        for provider in &data().providers {
            let path = format!("providers/{}", provider.logo);
            assert!(
                data_file_exists(&path),
                "provider '{}' logo '{}' is missing",
                provider.id,
                provider.logo
            );
            assert!(matches!(
                Path::new(&path)
                    .extension()
                    .and_then(|extension| extension.to_str()),
                Some("png" | "svg" | "webp")
            ));
        }
    }

    #[test]
    fn vendor_logos_and_homepage_flags_are_valid() {
        let featured = data()
            .vendors
            .iter()
            .filter(|vendor| vendor.homepage_featured)
            .map(|vendor| vendor.id.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            featured,
            HashSet::from(["alibaba", "deepseek", "mistral", "moonshot-ai", "z-ai",])
        );

        for vendor in &data().vendors {
            let Some(logo) = &vendor.logo else {
                assert!(!vendor.homepage_featured);
                continue;
            };
            let path = format!("models/{logo}");
            assert!(
                data_file_exists(&path),
                "vendor '{}' logo is missing",
                vendor.id
            );
            assert!(matches!(
                Path::new(&path)
                    .extension()
                    .and_then(|extension| extension.to_str()),
                Some("png" | "svg" | "webp")
            ));
        }
    }

    #[test]
    fn flagship_flags_belong_to_vendor_models() {
        let catalog = data();
        let mut flagships = catalog
            .vendors
            .iter()
            .flat_map(|vendor| &vendor.models)
            .filter(|model| model.flagship)
            .map(|model| model.id.as_str())
            .collect::<Vec<_>>();
        flagships.sort_unstable();
        assert_eq!(
            flagships,
            vec![
                "deepseek-v4.1-flash",
                "deepseek-v4.1-flash-1017",
                "glm-5.3",
                "kimi-k3",
                "mistral-large-4",
                "qwen3.8-2.4t-a95b",
                "qwen3.8-27b",
            ]
        );
        for model in catalog
            .vendors
            .iter()
            .flat_map(|vendor| &vendor.models)
            .filter(|model| model.flagship)
        {
            if let Some(url) = model.aa_report_url.as_deref() {
                assert!(
                    url.starts_with("https://artificialanalysis.ai/"),
                    "flagship model '{}' has an invalid AA report URL",
                    model.id
                );
            }
        }
    }
}
