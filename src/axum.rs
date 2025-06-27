// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! Module containing the function to add a metrics layer to axum
use axum::routing::Router;
use famedly_rust_utils::GenericCombinators;

use super::config::OtelConfig;

/// Adds a layer to create metrics if the metrics exporting is enabled
///
/// Example
///
/// This example shows how to call the macro. However, in this case, the layer
/// is not added because the default [`OtelConfig`] has the metrics exporting
/// disabled
///
/// ```rust
/// use axum::routing::{Router, get};
///
/// #[tokio::main]
/// async fn main() {
/// 	let config = Some(rust_telemetry::config::OtelConfig::default());
/// 	let app = Router::new().route("/", get("Test"));
/// 	let app = rust_telemetry::add_axum_metrics_layer!(app, config.as_ref());
///
/// 	let listener =
/// 		tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
/// 	let server = axum::serve(listener, app);
/// }
/// ```
#[macro_export]
macro_rules! add_axum_metrics_layer {
	($router:expr, $config:expr) => {
		$crate::axum::add_metrics_layer($router, $config, env!("CARGO_PKG_NAME"))
	};
}

#[allow(missing_docs)]
pub fn add_metrics_layer(
	router: Router,
	config: Option<&OtelConfig>,
	service_name: &'static str,
) -> Router {
	let enabled = config
		.and_then(|config| config.exporter.as_ref())
		.and_then(|exporter| exporter.metrics.as_ref())
		.is_some_and(|metrics| metrics.enabled);

	router.chain_if(enabled, |router| {
		let global_meter = opentelemetry::global::meter(service_name);
		let layer = tower_otel_http_metrics::HTTPMetricsLayerBuilder::builder()
			.with_meter(global_meter)
			.build()
			.inspect_err(|e| tracing::warn!("Error creating metrics layer: {e:?}"))
			.ok();
		router.chain_opt(layer, Router::layer)
	})
}
