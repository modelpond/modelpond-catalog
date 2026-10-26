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
}

impl fmt::Display for ProviderType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAi => write!(formatter, "openai"),
            Self::OpenRouter => write!(formatter, "openrouter"),
            Self::Anthropic => write!(formatter, "anthropic"),
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
    pub backend_enabled: bool,
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
    pub backend_enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderModel {
    pub model_id: String,
    pub vendor_model_id: String,
    pub api_model: String,
    #[serde(default)]
    pub flagship: bool,
    pub backend_enabled: bool,
    pub pricing: HostedPricing,
    pub serving_regions: Vec<String>,
    pub note: Option<String>,
    pub input_modalities: Option<Vec<String>>,
    pub output_modalities: Option<Vec<String>>,
    pub restricted_effort_settings: Option<EffortSettings>,
    #[serde(default)]
    pub profiles: Vec<SystemProfile>,
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
pub struct SystemProfile {
    pub id: i64,
    pub display_name: String,
    pub selectable: bool,
    pub legacy: bool,
    pub assigned_by: String,
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
    #[serde(rename = "type")]
    pub model_type: String,
    pub release_date: Option<String>,
    pub aa_score: Option<f64>,
    pub aa_report_url: Option<String>,
    pub blurb: String,
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

#[derive(Serialize)]
struct RetailProvider<'a> {
    id: &'a str,
    name: &'a str,
    logo: &'a str,
    country: &'a str,
    blurb: &'a str,
    urls: &'a BTreeMap<String, ProviderLink>,
    regions: Vec<RetailRegion<'a>>,
    backend_enabled: bool,
}

#[derive(Serialize)]
struct RetailRegion<'a> {
    id: &'a str,
    label: &'a str,
    description: Option<&'a str>,
    backend_enabled: bool,
}

#[derive(Serialize)]
struct RetailVendor<'a> {
    id: &'a str,
    name: &'a str,
    logo: Option<&'a str>,
    homepage_featured: bool,
    url: Option<&'a str>,
}

#[derive(Serialize)]
struct RetailBaseModel<'a> {
    id: &'a str,
    name: &'a str,
    company: &'a str,
    country: &'a str,
    #[serde(rename = "type")]
    model_type: &'a str,
    #[serde(rename = "releaseDate")]
    release_date: Option<&'a str>,
    #[serde(rename = "aaScore")]
    aa_score: Option<f64>,
    #[serde(rename = "aaReportUrl")]
    aa_report_url: Option<&'a str>,
    tagline: &'a str,
    input_modalities: &'a [String],
    output_modalities: &'a [String],
    vendor_id: &'a str,
    vendor_url: Option<&'a str>,
    vendor_blurb: Option<&'a str>,
    brand_logo: Option<&'a str>,
}

#[derive(Serialize)]
struct RetailHostedModel<'a> {
    model: String,
    provider: &'a str,
    #[serde(rename = "providerName")]
    provider_name: &'a str,
    #[serde(rename = "baseModel")]
    base_model: &'a str,
    #[serde(rename = "type")]
    model_type: &'a str,
    #[serde(skip_serializing_if = "is_false")]
    flagship: bool,
    input_modalities: Vec<String>,
    output_modalities: Vec<String>,
    currency: &'a str,
    #[serde(rename = "in")]
    input_price: &'a Price,
    #[serde(rename = "cachedIn", skip_serializing_if = "Option::is_none")]
    cached_input_price: Option<&'a Price>,
    #[serde(rename = "out")]
    output_price: &'a Price,
    #[serde(rename = "outNote", skip_serializing_if = "Option::is_none")]
    output_note: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<&'a str>,
    provider_id: &'a str,
    backend_enabled: bool,
    serving_regions: &'a [String],
    profile_ids: Vec<i64>,
    reasoning_efforts: Vec<String>,
    default_reasoning_effort: String,
    blurb: &'a str,
}

#[derive(Serialize)]
struct RetailProfile<'a> {
    id: i64,
    provider_id: &'a str,
    name: &'a str,
    model_name: &'a str,
    model_vendor: &'a str,
    model_vendor_url: Option<&'a str>,
    brand_logo: Option<&'a str>,
    reasoning_efforts: Vec<String>,
    default_reasoning_effort: String,
    hosted_model_key: String,
    assigned_by: &'a str,
    legacy: bool,
    selectable: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
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

    pub fn vendor_model(&self, model_id: &str) -> Option<(&Vendor, &VendorModel)> {
        self.vendors.iter().find_map(|vendor| {
            vendor
                .models
                .iter()
                .find(|model| model.id == model_id)
                .map(|model| (vendor, model))
        })
    }

    pub fn backend_region<'a>(
        &'a self,
        provider: &'a Provider,
        model: &'a ProviderModel,
    ) -> Option<&'a ProviderRegion> {
        provider.regions.iter().find(|region| {
            region.backend_enabled && model.serving_regions.iter().any(|id| id == &region.id)
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
        let mut profile_ids = HashSet::new();
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
                if region.backend_enabled && !provider.backend_enabled {
                    return Err(format!(
                        "region '{}:{}' is backend-enabled under a disabled provider",
                        provider.id, region.id
                    ));
                }
                if region.backend_enabled
                    && (region.endpoint.is_none()
                        || region.api_convention.is_none()
                        || region.api_key.is_none())
                {
                    return Err(format!(
                        "backend region '{}:{}' is missing API configuration",
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
                if model.serving_regions.is_empty() {
                    return Err(format!(
                        "provider model '{model_label}' has no serving regions"
                    ));
                }
                if self.vendor_model(&model.vendor_model_id).is_none() {
                    return Err(format!(
                        "provider model '{model_label}' references unknown vendor model '{}'",
                        model.vendor_model_id
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
                if model.backend_enabled && self.backend_region(provider, model).is_none() {
                    return Err(format!(
                        "backend model '{model_label}' has no enabled serving region"
                    ));
                }
                if !model.profiles.is_empty() && !model.backend_enabled {
                    return Err(format!(
                        "model with profiles '{model_label}' is not backend-enabled"
                    ));
                }
                if let Some(settings) = &model.restricted_effort_settings {
                    validate_effort_settings(settings, &model_label, &effort_levels)?;
                }
                for profile in &model.profiles {
                    if profile.id >= 0 {
                        return Err(format!(
                            "catalog profile '{}' must have a negative id",
                            profile.id
                        ));
                    }
                    if profile.assigned_by != "system" {
                        return Err(format!(
                            "catalog profile '{}' must be system-assigned",
                            profile.id
                        ));
                    }
                    if !profile_ids.insert(profile.id) {
                        return Err(format!("duplicate profile id '{}'", profile.id));
                    }
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
                if !model_ids.insert(model.id.as_str()) {
                    return Err(format!("duplicate model id '{}'", model.id));
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

pub fn export_retail_js() -> anyhow::Result<String> {
    let catalog = data();
    let retail_helpers = read_data_file("catalog-retail-helpers.js")?;
    let providers = catalog
        .providers
        .iter()
        .map(|provider| RetailProvider {
            id: &provider.id,
            name: &provider.name,
            logo: &provider.logo,
            country: &provider.country,
            blurb: &provider.blurb,
            urls: &provider.urls,
            regions: provider
                .regions
                .iter()
                .map(|region| RetailRegion {
                    id: &region.id,
                    label: &region.label,
                    description: region.description.as_deref(),
                    backend_enabled: region.backend_enabled,
                })
                .collect(),
            backend_enabled: provider.backend_enabled,
        })
        .collect::<Vec<_>>();

    let vendors = catalog
        .vendors
        .iter()
        .map(|vendor| RetailVendor {
            id: &vendor.id,
            name: &vendor.name,
            logo: vendor.logo.as_deref(),
            homepage_featured: vendor.homepage_featured,
            url: vendor.url.as_deref(),
        })
        .collect::<Vec<_>>();

    let base_models = catalog
        .vendors
        .iter()
        .flat_map(|vendor| {
            vendor.models.iter().map(|model| RetailBaseModel {
                id: &model.id,
                name: &model.name,
                company: &vendor.name,
                country: &vendor.country,
                model_type: &model.model_type,
                release_date: model.release_date.as_deref(),
                aa_score: model.aa_score,
                aa_report_url: model.aa_report_url.as_deref(),
                tagline: &model.blurb,
                input_modalities: &model.input_modalities,
                output_modalities: &model.output_modalities,
                vendor_id: &vendor.id,
                vendor_url: vendor.url.as_deref(),
                vendor_blurb: vendor.blurb.as_deref(),
                brand_logo: vendor.logo.as_deref(),
            })
        })
        .collect::<Vec<_>>();

    let mut hosted_models = Vec::new();
    let mut profiles = Vec::new();
    for provider in &catalog.providers {
        for hosted in &provider.models {
            let (vendor, model) =
                catalog
                    .vendor_model(&hosted.vendor_model_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "provider model '{}/{}' references unknown vendor model '{}'",
                            provider.id,
                            hosted.model_id,
                            hosted.vendor_model_id
                        )
                    })?;
            let input_modalities = hosted
                .input_modalities
                .as_ref()
                .unwrap_or(&model.input_modalities)
                .clone();
            let output_modalities = hosted
                .output_modalities
                .as_ref()
                .unwrap_or(&model.output_modalities)
                .clone();
            let effort_settings = hosted.effort_settings(model);
            let efforts = effort_settings.supported.clone();
            let default_effort = effort_settings.default.clone();

            for region_id in &hosted.serving_regions {
                let region = provider
                    .regions
                    .iter()
                    .find(|region| &region.id == region_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "provider model '{}/{}' references unknown region '{}'",
                            provider.id,
                            hosted.model_id,
                            region_id
                        )
                    })?;
                let backend_enabled = hosted.backend_enabled && region.backend_enabled;

                hosted_models.push(RetailHostedModel {
                    model: provider_model_key(&provider.id, region_id, &hosted.model_id),
                    provider: &provider.name,
                    provider_name: &hosted.api_model,
                    base_model: &hosted.vendor_model_id,
                    model_type: &model.model_type,
                    flagship: hosted.flagship,
                    input_modalities: input_modalities.clone(),
                    output_modalities: output_modalities.clone(),
                    currency: &hosted.pricing.currency,
                    input_price: &hosted.pricing.input,
                    cached_input_price: hosted.pricing.cached_input.as_ref(),
                    output_price: &hosted.pricing.output,
                    output_note: hosted.pricing.output_note.as_deref(),
                    note: hosted.note.as_deref(),
                    provider_id: &provider.id,
                    backend_enabled,
                    serving_regions: std::slice::from_ref(region_id),
                    profile_ids: if backend_enabled {
                        hosted.profiles.iter().map(|profile| profile.id).collect()
                    } else {
                        Vec::new()
                    },
                    reasoning_efforts: efforts.clone(),
                    default_reasoning_effort: default_effort.clone(),
                    blurb: &model.blurb,
                });
            }

            if !hosted.profiles.is_empty() {
                let profile_region = catalog.backend_region(provider, hosted).ok_or_else(|| {
                    anyhow::anyhow!(
                        "model with profiles '{}/{}' has no backend region",
                        provider.id,
                        hosted.model_id
                    )
                })?;
                let hosted_model_key =
                    provider_model_key(&provider.id, &profile_region.id, &hosted.model_id);
                profiles.extend(hosted.profiles.iter().map(|profile| RetailProfile {
                    id: profile.id,
                    provider_id: &provider.id,
                    name: &profile.display_name,
                    model_name: &hosted.api_model,
                    model_vendor: vendor.full_name.as_deref().unwrap_or(&vendor.name),
                    model_vendor_url: vendor.url.as_deref(),
                    brand_logo: vendor.logo.as_deref(),
                    reasoning_efforts: efforts.clone(),
                    default_reasoning_effort: default_effort.clone(),
                    hosted_model_key: hosted_model_key.clone(),
                    assigned_by: &profile.assigned_by,
                    legacy: profile.legacy,
                    selectable: profile.selectable,
                }));
            }
        }
    }

    Ok(format!(
        "// Generated from Model Pond's backend catalog. Do not edit directly.\n\
         const CATALOG_SCHEMA_VERSION = {};\n\
         const CATALOG_PRICING_POLICY = {};\n\
         const BASE_MODELS = {};\n\
         const CATALOG = {};\n\
         const CATALOG_VENDORS = {};\n\
         const CATALOG_PROVIDERS = {};\n\
         const CATALOG_PROFILES = {};\n\
         const CATALOG_SECTIONS = {};\n\
         const CATALOG_ICONS = {};\n\
         const CATALOG_ORDER = {};\n\n{}",
        serde_json::to_string(&catalog.config.version)?,
        serde_json::to_string_pretty(&catalog.config.pricing_policy)?,
        serde_json::to_string_pretty(&base_models)?,
        serde_json::to_string_pretty(&hosted_models)?,
        serde_json::to_string_pretty(&vendors)?,
        serde_json::to_string_pretty(&providers)?,
        serde_json::to_string_pretty(&profiles)?,
        serde_json::to_string_pretty(&catalog.config.sections)?,
        serde_json::to_string_pretty(&catalog.config.icons)?,
        serde_json::to_string_pretty(&catalog.config.modality_order)?,
        retail_helpers,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            163
        );
        assert_eq!(catalog.vendors.len(), 24);
        assert_eq!(
            catalog
                .vendors
                .iter()
                .map(|vendor| vendor.models.len())
                .sum::<usize>(),
            88
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
        assert_eq!(model.model_type, "Text");
        assert_eq!(hosted.pricing.currency, "EUR");
        assert_eq!(hosted.pricing.output.value, Some(3.3));
        assert_eq!(model.aa_score, Some(33.7));
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
    fn flagship_models_link_to_artificial_analysis_reports() {
        let catalog = data();
        for model_id in [
            "qwen3.8-27b",
            "deepseek-v4-flash-0731",
            "mistral-large-4",
            "kimi-k3",
            "glm-5.3",
        ] {
            let (_, model) = catalog
                .vendor_model(model_id)
                .expect("flagship model should exist");
            assert!(
                model
                    .aa_report_url
                    .as_deref()
                    .is_some_and(|url| url.starts_with("https://artificialanalysis.ai/")),
                "flagship model '{model_id}' needs an AA report URL"
            );
        }
    }

    #[test]
    fn system_profiles_keep_negative_ids() {
        let profiles = data()
            .providers
            .iter()
            .flat_map(|provider| &provider.models)
            .flat_map(|model| &model.profiles)
            .collect::<Vec<_>>();
        assert!(!profiles.is_empty());
        assert!(profiles.iter().all(|profile| profile.id < 0));
        assert!(
            profiles
                .iter()
                .all(|profile| profile.assigned_by == "system")
        );
    }

    #[test]
    fn retail_export_contains_combined_metadata_without_backend_access() {
        let api_json = serde_json::to_string(data()).expect("catalog API should serialize");
        let script = export_retail_js().expect("catalog should export");
        assert!(script.contains("const BASE_MODELS ="));
        assert!(script.contains("const CATALOG ="));
        assert!(script.contains("const CATALOG_VENDORS ="));
        assert!(script.contains("const CATALOG_PROVIDERS ="));
        assert!(script.contains("const CATALOG_PROFILES ="));
        assert!(script.contains("const CATALOG_PRICING_POLICY ="));
        assert!(script.contains("function catalogModalities"));
        assert!(script.contains("\"input_modalities\""));
        assert!(script.contains("\"output_modalities\""));
        assert!(script.contains("\"privacy\""));
        assert!(script.contains("\"zdr\""));
        assert!(script.contains("\"currency\": \"USD\""));
        assert!(!script.contains("\"capabilities\""));
        assert!(!script.contains("api_key"));
        assert!(!script.contains("api_base_url"));
        assert!(!script.contains("https://api.scaleway.ai/v1"));
        assert!(!api_json.contains("SCALEWAY_API_KEY"));
        assert!(!api_json.contains("https://api.scaleway.ai/v1"));
    }
}
