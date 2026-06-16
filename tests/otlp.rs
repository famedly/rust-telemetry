// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for OTLP

#![allow(clippy::expect_used)]

use axum::routing::post;
use rust_telemetry::{
	config::{ExporterConfig, OtelConfig, StdoutLogsConfig},
	init_otel,
};
use url::Url;

/// The current version of the OTLP exporter also induces traces after it's been
/// shut down, causing a feedback loop which ends in a stack overflow
///
/// This test checks that we can start rust-telemetry with an OTLP endpoint
/// enabled, output a log line, and exit our program without a stack overflow.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_otlp_exporter() {
	let router = post(|| async { Vec::new() });
	let listener =
		tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Could not bind TCPListener");
	let endpoint = Url::parse(&format!(
		"http://{}",
		listener.local_addr().expect("Failed getting address for listener")
	))
	.expect("Failed parsing URL for endpoint");
	let _mock_otlp = tokio::spawn(async move { axum::serve(listener, router).await });

	let provider_config = Some(rust_telemetry::config::ProviderConfig {
		enabled: true,
		general_level: famedly_rust_utils::LevelFilter(tracing::level_filters::LevelFilter::DEBUG),
		..Default::default()
	});

	let config = OtelConfig {
		stdout: Some(StdoutLogsConfig { enabled: true, ..Default::default() }),
		exporter: Some(ExporterConfig {
			endpoint: endpoint.into(),
			logs: provider_config.clone(),
			traces: provider_config.clone(),
			..Default::default()
		}),
	};
	let _guard = init_otel!(&config).expect("Error initializing Otel");
	tracing::info!("Output a line");
}
