//! OpenTelemetry initialization
//!
//! Lib containing the definitions and initializations of the OpenTelemetry
//! tools
use std::str::FromStr as _;

use config::{OtelConfig, StdoutLogsConfig};
use opentelemetry::{
	trace::{TraceError, TracerProvider as _},
	KeyValue,
};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter, WithExportConfig as _};
use opentelemetry_sdk::{
	logs::{LogError, LoggerProvider},
	metrics::{MeterProviderBuilder, MetricError, PeriodicReader, SdkMeterProvider},
	propagation::TraceContextPropagator,
	runtime,
	trace::{RandomIdGenerator, TracerProvider},
	Resource,
};
use opentelemetry_semantic_conventions::{
	resource::{SERVICE_NAME, SERVICE_VERSION},
	SCHEMA_URL,
};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{
	layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter, Layer,
};
use url::Url;

#[cfg(feature = "axum")]
pub mod axum;
pub mod config;
#[cfg(feature = "reqwest-middleware")]
pub mod reqwest_middleware;

/// Crates a resource for the Otel providers
fn resource(service_name: String, version: String) -> Resource {
	Resource::from_schema_url(
		[KeyValue::new(SERVICE_NAME, service_name), KeyValue::new(SERVICE_VERSION, version)],
		SCHEMA_URL,
	)
}

/// Setup a Otel exporter and a provider for traces
fn init_traces(
	endpoint: Url,
	service_name: String,
	version: String,
) -> Result<TracerProvider, TraceError> {
	let exporter = SpanExporter::builder().with_tonic().with_endpoint(endpoint).build()?;
	let tracer_provider = TracerProvider::builder()
		.with_id_generator(RandomIdGenerator::default())
		.with_resource(resource(service_name, version))
		.with_batch_exporter(exporter, runtime::Tokio)
		.build();

	opentelemetry::global::set_tracer_provider(tracer_provider.clone());
	Ok(tracer_provider)
}

/// Setup a Otel exporter and a provider for metrics
fn init_metrics(
	endpoint: Url,
	service_name: String,
	version: String,
) -> Result<SdkMeterProvider, MetricError> {
	let exporter = opentelemetry_otlp::MetricExporter::builder()
		.with_tonic()
		.with_endpoint(endpoint)
		.with_temporality(opentelemetry_sdk::metrics::Temporality::default())
		.build()?;

	let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();

	let meter_provider = MeterProviderBuilder::default()
		.with_resource(resource(service_name, version))
		.with_reader(reader)
		.build();

	opentelemetry::global::set_meter_provider(meter_provider.clone());
	Ok(meter_provider)
}

/// Setup a Otel exporter and a provider for logs
fn init_logs(
	endpoint: Url,
	service_name: String,
	version: String,
) -> Result<LoggerProvider, LogError> {
	let exporter = LogExporter::builder().with_tonic().with_endpoint(endpoint).build()?;

	Ok(LoggerProvider::builder()
		.with_resource(resource(service_name, version))
		.with_batch_exporter(exporter, runtime::Tokio)
		.build())
}

/// Initializes the OpenTelemetry
///
/// example
/// ```rust
/// use rust_telemetry::config;
///
/// #[tokio::main]
/// async fn main() {
/// 	let _guard = rust_telemetry::init_otel(
/// 		&config::OtelConfig::default(),
/// 		env!("CARGO_CRATE_NAME"),
/// 	);
///
/// 	// ...
/// }
/// ```
#[must_use = "The return is a guard for the providers and it need to be kept to properly shutdown them"]
pub fn init_otel(
	config: &OtelConfig,
	main_crate: &'static str,
) -> Result<ProvidersGuard, OtelInitError> {
	opentelemetry::global::set_text_map_propagator(TraceContextPropagator::default());

	let stdout_layer = config
		.stdout
		.as_ref()
		.or(Some(&StdoutLogsConfig::default()))
		.and_then(|stdout| stdout.enabled.then_some(stdout))
		.map(|logger_config| {
			let filter_fmt = EnvFilter::from_str(&logger_config.get_filter(main_crate))?;
			Ok::<_, OtelInitError>(
				tracing_subscriber::fmt::layer().with_thread_names(true).with_filter(filter_fmt),
			)
		})
		.transpose()?;

	let (logger_provider, logs_layer) = config
		.exporter
		.as_ref()
		.and_then(|exporter| {
			exporter.logs.as_ref().and_then(|c| c.enabled.then_some(c)).map(|logger_config| {
				let filter_otel = EnvFilter::from_str(&logger_config.get_filter(main_crate))?;
				let logger_provider = init_logs(
					exporter.get_endpoint(),
					exporter.service_name.clone(),
					exporter.version.clone(),
				)?;

				// Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
				let logs_layer = OpenTelemetryTracingBridge::new(&logger_provider);
				let logs_layer = logs_layer.with_filter(filter_otel);

				Ok::<_, OtelInitError>((Some(logger_provider), Some(logs_layer)))
			})
		})
		.transpose()?
		.unwrap_or((None, None));

	let (tracer_provider, tracer_layer) = config
		.exporter
		.as_ref()
		.and_then(|exporter| {
			exporter.traces.as_ref().and_then(|c| c.enabled.then_some(c)).map(|tracer_config| {
				let trace_filter = EnvFilter::from_str(&tracer_config.get_filter(main_crate))?;
				let tracer_provider = init_traces(
					exporter.get_endpoint(),
					exporter.service_name.clone(),
					exporter.version.clone(),
				)?;
				let tracer = tracer_provider.tracer(exporter.service_name.clone());
				let tracer_layer = OpenTelemetryLayer::new(tracer).with_filter(trace_filter);
				Ok::<_, OtelInitError>((Some(tracer_provider), Some(tracer_layer)))
			})
		})
		.transpose()?
		.unwrap_or((None, None));

	let (meter_provider, meter_layer) = config
		.exporter
		.as_ref()
		.and_then(|exporter| {
			exporter.metrics.as_ref().and_then(|c| c.enabled.then_some(c)).map(|meter_config| {
				let metrics_filter = EnvFilter::from_str(&meter_config.get_filter(main_crate))?;
				let meter_provider = init_metrics(
					exporter.get_endpoint(),
					exporter.service_name.clone(),
					exporter.version.clone(),
				)?;
				let meter_layer =
					MetricsLayer::new(meter_provider.clone()).with_filter(metrics_filter);

				Ok::<_, OtelInitError>((Some(meter_provider), Some(meter_layer)))
			})
		})
		.transpose()?
		.unwrap_or((None, None));

	// Initialize the tracing subscriber with the stdout layer and
	// layers for exporting over OpenTelemetry the logs, traces and metrics.
	tracing_subscriber::registry()
		.with(logs_layer)
		.with(stdout_layer)
		.with(meter_layer)
		.with(tracer_layer)
		.init();

	Ok(ProvidersGuard { logger_provider, tracer_provider, meter_provider })
}

/// Guarding object to make sure the providers are properly shutdown
#[derive(Debug)]
pub struct ProvidersGuard {
	/// Logger provider
	logger_provider: Option<LoggerProvider>,
	/// Tracer provider
	tracer_provider: Option<TracerProvider>,
	/// Meter provider
	meter_provider: Option<SdkMeterProvider>,
}

// Necessary to call TracerProvider::shutdown() on exit
// due to a bug with flushing on global shutdown:
// https://github.com/open-telemetry/opentelemetry-rust/issues/1961
impl Drop for ProvidersGuard {
	fn drop(&mut self) {
		// This causes a hang in testing.
		// Some relevant information:
		// https://github.com/open-telemetry/opentelemetry-rust/issues/536
		#[cfg(not(test))]
		{
			if let Some(logger_provider) = self.logger_provider.as_ref() {
				let _ = logger_provider.shutdown().inspect_err(|err| {
					tracing::error!("Could not shutdown LoggerProvider: {err}");
				});
			}
			if let Some(tracer_provider) = self.tracer_provider.as_ref() {
				let _ = tracer_provider.shutdown().inspect_err(|err| {
					tracing::error!("Could not shutdown TracerProvider: {err}");
				});
			}
			if let Some(meter_provider) = self.meter_provider.as_ref() {
				let _ = meter_provider.shutdown().inspect_err(|err| {
					tracing::error!("Could not shutdown MeterProvider: {err}");
				});
			}
		}
	}
}

/// OpenTelemetry setup errors
#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum OtelInitError {
	#[error("Logger initialization error: {0}")]
	LoggerInitError(#[from] LogError),
	#[error("Tracer initialization error: {0}")]
	TracerInitError(#[from] TraceError),
	#[error("Meter initialization error: {0}")]
	MeterInitError(#[from] MetricError),
	#[error("Parsing EnvFilter directives error: {0}")]
	EnvFilterError(#[from] tracing_subscriber::filter::ParseError),
}

#[cfg(test)]
mod tests {
	#![allow(clippy::expect_used)]
	use super::{
		config::{ExporterConfig, OtelConfig, ProviderConfig},
		init_otel,
	};
	use crate::config::StdoutLogsConfig;

	#[tokio::test]
	async fn test_tracer_provider_enabled() {
		let config = OtelConfig {
			stdout: None,
			exporter: Some(ExporterConfig {
				traces: Some(ProviderConfig { enabled: true, ..Default::default() }),
				..Default::default()
			}),
		};
		let guard = init_otel(&config, env!("CARGO_PKG_NAME")).expect("Error initializing Otel");
		assert!(guard.tracer_provider.is_some());
	}
	#[tokio::test]
	async fn test_tracer_provider_disabled() {
		let config_enabled_false = OtelConfig {
			stdout: None,
			exporter: Some(ExporterConfig {
				traces: Some(ProviderConfig { enabled: false, ..Default::default() }),
				..Default::default()
			}),
		};
		let guard = init_otel(&config_enabled_false, env!("CARGO_PKG_NAME"))
			.expect("Error initializing Otel");
		assert!(guard.tracer_provider.is_none());
	}

	// There seems to be a problem when testing the scenario when the meter
	// provider is enable. The tests hangs when calling the shutdown from the
	// PeriodicReader. For now we won't test this scenarios
	//https://github.com/open-telemetry/opentelemetry-rust/issues/2056

	#[tokio::test]
	async fn test_meter_provider_disabled() {
		let config_enabled_false = OtelConfig {
			stdout: None,
			exporter: Some(ExporterConfig {
				metrics: Some(ProviderConfig { enabled: false, ..Default::default() }),
				..Default::default()
			}),
		};
		let guard = init_otel(&config_enabled_false, env!("CARGO_PKG_NAME"))
			.expect("Error initializing Otel");
		assert!(guard.meter_provider.is_none());
	}
	#[tokio::test]
	async fn test_logger_provider_enabled() {
		let config = OtelConfig {
			stdout: None,
			exporter: Some(ExporterConfig {
				logs: Some(ProviderConfig { enabled: true, ..Default::default() }),
				..Default::default()
			}),
		};
		let guard = init_otel(&config, env!("CARGO_PKG_NAME")).expect("Error initializing Otel");
		assert!(guard.logger_provider.is_some());
	}
	#[tokio::test]
	async fn test_logger_provider_disabled() {
		let config_enabled_false = OtelConfig {
			stdout: None,
			exporter: Some(ExporterConfig {
				logs: Some(ProviderConfig { enabled: false, ..Default::default() }),
				..Default::default()
			}),
		};
		let guard = init_otel(&config_enabled_false, env!("CARGO_PKG_NAME"))
			.expect("Error initializing Otel");
		assert!(guard.logger_provider.is_none());
	}

	#[tokio::test]
	async fn test_exporter_config_none() {
		let config_none = OtelConfig {
			stdout: Some(StdoutLogsConfig { enabled: true, ..Default::default() }),
			exporter: Some(ExporterConfig::default()),
		};
		let guard =
			init_otel(&config_none, env!("CARGO_PKG_NAME")).expect("Error initializing Otel");
		assert!(guard.meter_provider.is_none());
		assert!(guard.tracer_provider.is_none());
		assert!(guard.logger_provider.is_none());
	}
}
