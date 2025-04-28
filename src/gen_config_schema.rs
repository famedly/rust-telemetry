// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! Generates config schema for this crate:
//! ```sh
//! cargo run --features serde_yaml,schemars --bin gen-config-schema > config-schema.yaml
//! ```
#![allow(clippy::unwrap_used, clippy::print_stdout)]

fn main() {
	let schema = schemars::schema_for!(rust_telemetry::config::OtelConfig);
	print!("{}", serde_yaml::to_string(&schema).unwrap());
}
