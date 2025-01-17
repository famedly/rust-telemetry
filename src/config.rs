//! OpenTelemetry configuration
//!
//! Module containing the configuration struct for the OpenTelemetry
use std::str::FromStr as _;

use famedly_rust_utils::LevelFilter;
use serde::Deserialize;
use url::Url;

/// Default gRPC Otel endpoint
const DEFAULT_ENDPOINT: &str = "http://localhost:4317";

/// OpenTelemetry configuration
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OtelConfig {
	/// Enables logs on stdout
	pub stdout: Option<StdoutLogsConfig>,
	/// Configurations for exporting traces, metrics and logs
	pub exporter: Option<ExporterConfig>,
}

/// Configuration for exporting OpenTelemetry data
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ExporterConfig {
	/// gRPC endpoint for exporting using OTELP
	pub endpoint: Option<Url>,
	/// Application service name
	pub service_name: String,
	/// Application version
	pub version: String,

	/// Logs exporting config
	pub logs: Option<ProviderConfig>,
	/// Traces exporting config
	pub traces: Option<ProviderConfig>,
	/// Metrics exporting config
	pub metrics: Option<ProviderConfig>,
}

/// Stdout logs configuration
#[derive(Debug, Deserialize, Clone)]
pub struct StdoutLogsConfig {
	/// Enables the stdout logs
	pub enabled: bool,
	/// Level for the crate
	#[serde(default = "default_level_filter")]
	pub level: LevelFilter,
	/// Level for the dependencies
	#[serde(default = "default_level_filter")]
	pub general_level: LevelFilter,
}

/// Provider configuration for OpenTelemetry export
#[derive(Debug, Deserialize, Clone)]
pub struct ProviderConfig {
	/// Enables provider
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
		Self { enabled: true, level: default_level_filter(), general_level: default_level_filter() }
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

impl ExporterConfig {
	#[allow(clippy::expect_used)]
	/// Gets the configured exporting endpoint or uses the default one
	pub(crate) fn get_endpoint(&self) -> Url {
		self.endpoint
			.clone()
			.unwrap_or(Url::from_str(DEFAULT_ENDPOINT).expect("Error parsing default endpoint"))
	}
}

/// Sets the default LevelFilter
const fn default_level_filter() -> LevelFilter {
	LevelFilter(tracing::level_filters::LevelFilter::INFO)
}
