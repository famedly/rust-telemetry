// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! Reqwest Middleware
//!
//! This module provides a reqwest middleware that will propagate the
//! OpenTelemetry current context by setting the appropriated headers on the
//! request. This layer should be used with the crate [`reqwest_middleware`]

use http::Extensions;
use opentelemetry_http::HeaderInjector;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

/// Middleware for [`reqwest_middleware`] to propagate the Otel context
///
/// Example
///
/// ```rust
/// use rust_telemetry::reqwest_middleware::OtelMiddleware;
///
/// #[tokio::main]
/// async fn main() {
/// 	let reqwest_client = reqwest::Client::builder().build().unwrap();
/// 	let client = reqwest_middleware::ClientBuilder::new(reqwest_client)
/// 		// Insert the tracing middleware
/// 		.with(OtelMiddleware::default())
/// 		.build();
/// 	client.get("http://localhost").send().await;
/// }
/// ```
#[derive(Debug, Default)]
pub struct OtelMiddleware;

#[async_trait::async_trait]
impl Middleware for OtelMiddleware {
	async fn handle(
		&self,
		mut req: Request,
		extensions: &mut Extensions,
		next: Next<'_>,
	) -> Result<Response> {
		opentelemetry::global::get_text_map_propagator(|propagator| {
			let cx = Span::current().context();
			propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut()));
		});
		next.run(req, extensions).await
	}
}
