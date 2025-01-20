//! Module containing the function to add a metrics layer to axum
use axum::routing::Router;

use super::config::OtelConfig;

/// Adds a layer to create metrics if the metrics exporting is enabled
///
/// Example
///
/// This example shows how to call the add_metrics_layer however, in this case,
/// the layer is not added because the default OtelConfig has the metrics
/// exporting disabled
///
/// ```rust
/// use axum::routing::{get, Router};
///
/// #[tokio::main]
/// async fn main() {
/// 	let config = Some(rust_telemetry::config::OtelConfig::default());
/// 	let app = Router::new().route("/", get("Test"));
/// 	let app = rust_telemetry::axum::add_metrics_layer(app, config);
///
/// 	let listener =
/// 		tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
/// 	let server = axum::serve(listener, app);
/// }
/// ```
pub fn add_metrics_layer(router: Router, config: Option<OtelConfig>) -> Router {
	let mut router = router;
	if let Some(exporter) = config.and_then(|config| config.exporter) {
		if exporter.metrics.is_some_and(|config| config.enabled) {
			let global_meter =
				opentelemetry::global::meter(Box::leak(exporter.service_name.into_boxed_str()));
			router = match tower_otel_http_metrics::HTTPMetricsLayerBuilder::new()
				.with_meter(global_meter)
				.build()
			{
				Ok(layer) => router.layer(layer),
				Err(e) => {
					tracing::warn!("Error creating metrics layer. {e:?}");
					router
				}
			};
		}
	}
	router
}
