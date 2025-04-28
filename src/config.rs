// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry configuration
//!
//! Module containing the configuration struct for the OpenTelemetry

use std::collections::BTreeMap as Map;

use famedly_rust_utils::LevelFilter;
use serde::Deserialize;
use url::Url;

/// Default gRPC Otel endpoint
const DEFAULT_ENDPOINT: &str = "http://localhost:4317";

/// Wrapper over [`Url`] with [`Default`] implementation `http://localhost:4317`
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
#[allow(missing_docs)]
pub struct OtelUrl {
	pub url: Url,
}

impl From<Url> for OtelUrl {
	fn from(url: Url) -> Self {
		Self { url }
	}
}

#[allow(clippy::expect_used)]
impl Default for OtelUrl {
	fn default() -> Self {
		Self { url: Url::parse(DEFAULT_ENDPOINT).expect("Error parsing default endpoint") }
	}
}

/// OpenTelemetry configuration
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct OtelConfig {
	/// Enables logs on stdout
	pub stdout: Option<StdoutLogsConfig>,
	/// Configurations for exporting traces, metrics and logs
	pub exporter: Option<ExporterConfig>,
}

impl OtelConfig {
	/// Helper constructor to get stdout-only config for use in tests.
	#[must_use]
	pub fn for_tests() -> Self {
		OtelConfig {
			stdout: Some(StdoutLogsConfig {
				enabled: true,
				level: tracing_subscriber::filter::LevelFilter::TRACE.into(),
				general_level: tracing_subscriber::filter::LevelFilter::INFO.into(),
				json_output: false,
			}),
			exporter: None,
		}
	}
}

/// Configuration for exporting OpenTelemetry data
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExporterConfig {
	/// gRPC endpoint for exporting using OTELP
	#[serde(default)]
	pub endpoint: OtelUrl,
	/// Key value mapping of the OTEL resource. See [Resource semantic conventions](https://opentelemetry.io/docs/specs/semconv/resource/) for what can be set here.
	/// Only string values are supported now.
	/// This crate sets `service.name` and `service.version` by default.
	#[serde(default)]
	pub resource_metadata: Map<String, String>,
	/// Logs exporting config
	pub logs: Option<ProviderConfig>,
	/// Traces exporting config
	pub traces: Option<ProviderConfig>,
	/// Metrics exporting config
	pub metrics: Option<ProviderConfig>,
}

/// Stdout logs configuration
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct StdoutLogsConfig {
	/// Enables the stdout logs
	#[serde(default = "true_")]
	pub enabled: bool,
	/// Level for the crate
	#[serde(default = "default_level_filter")]
	pub level: LevelFilter,
	/// Level for the dependencies
	#[serde(default = "default_level_filter")]
	pub general_level: LevelFilter,
	/// Output structured JSON logs
	#[serde(default)]
	pub json_output: bool,
}

/// Provider configuration for OpenTelemetry export
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
	/// Enables provider
	#[serde(default)]
	pub enabled: bool,
	/// Level for the crate
	#[serde(default = "default_level_filter")]
	pub level: LevelFilter,
	/// Level for the dependencies
	#[serde(default = "default_level_filter")]
	pub general_level: LevelFilter,
}

impl ProviderConfig {
	/// Builds a trace filter
	pub(crate) fn get_filter(&self, crate_name: &'static str) -> String {
		format!("{},{}={}", self.general_level, crate_name, self.level)
	}
}

impl StdoutLogsConfig {
	/// Builds a trace filter
	pub(crate) fn get_filter(&self, crate_name: &'static str) -> String {
		format!("{},{}={}", self.general_level, crate_name, self.level)
	}
}

impl Default for StdoutLogsConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			level: default_level_filter(),
			general_level: default_level_filter(),
			json_output: false,
		}
	}
}

impl Default for ProviderConfig {
	fn default() -> Self {
		Self {
			enabled: false,
			level: default_level_filter(),
			general_level: default_level_filter(),
		}
	}
}

/// Sets the default LevelFilter
const fn default_level_filter() -> LevelFilter {
	LevelFilter(tracing::level_filters::LevelFilter::INFO)
}

/// Workaround for [serde-rs/serde#368](https://github.com/serde-rs/serde/issues/368)
const fn true_() -> bool {
	true
}
