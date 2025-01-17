# Rust Telemetry

[![rust workflow status][badge-rust-workflow-img]][badge-rust-workflow-url]
[![docker workflow status][badge-docker-workflow-img]][badge-docker-workflow-url]
[![docs main][badge-docs-main-img]][badge-docs-main-url]

[badge-rust-workflow-img]: https://github.com/famedly/rust-library-template/actions/workflows/rust.yml/badge.svg
[badge-rust-workflow-url]: https://github.com/famedly/rust-library-template/commits/main
[badge-docker-workflow-img]: https://github.com/famedly/rust-library-template/actions/workflows/docker.yml/badge.svg
[badge-docker-workflow-url]: https://github.com/famedly/rust-library-template/commits/main
[badge-docs-main-img]: https://img.shields.io/badge/docs-main-blue
[badge-docs-main-url]: https://famedly.github.io/rust-library-template/project_name/index.html


This lib contains a set of helpers to work with OpenTelemetry logs, traces and metrics.

### Setup

For setup all that's needed it to run the function `rust_telemetry::init_otel`. The function returns a guard that takes care of properly shutting down the providers.

If no configuration is present the exporting of logs traces and metrics is disabled and the stdout logging is enabled.

The functions on the crate exporting opentelemetry traces should be annotated with `tracing::instrument` to generate a new span for that function. Documentation on this macro can be found on the [here](https://docs.rs/tracing/latest/tracing/attr.instrument.html)

The opentelemetry information is exported using gRPC to and opentelemetry collector. By default the expected endpoint is `http://localhots:4317`

The default level of logging and traces is `info` for the crate and all it's dependencies. This level can be changed through the configuration and, the result filter expression is `general_level,main_crate=level` where `general_level` and `level` come from the configuration and `main_crate` is an argument for the `init_otel` function

```rust
#[tokio::main]
async fn main() {
  let _guard = init_otel(&config, env!("CARGO_CRATE_NAME")).unwrap();

}
```


### Propagate the context

A context can be propagated to allow linking the traces from two different services. This is done by injecting the context information into the request and retrieving it in another service.

#### reqwest

For injecting the current context using the reqwest client we can wrap a client in a [reqwest-middleware](https://crates.io/crates/reqwest-middleware) and use the `OtelMiddleware` middleware present in this crate. This feature requires the feature flag `reqwest-middleware`

```rust
use rust_telemetry::reqwest_middleware::OtelMiddleware;

let reqwest_client = reqwest::Client::builder().build().unwrap();
let client = reqwest_middleware::ClientBuilder::new(reqwest_client)
  // Insert the tracing middleware
  .with(OtelMiddleware::default())
  .build();
client.get("http://localhost").send().await;
```

### axum

For retrieving a context using axum we can use the `OtelAxumLayer` from [axum_tracing_opentelemetry](https://crates.io/crates/axum-tracing-opentelemetry)

> [!WARNING]
> This only seems to be working using the feature flag `tracing_level_info`. See the [issue](https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/issues/148)

This layer should run as soon as possible

```rust
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;

Router::new().layer(OtelAxumLayer::default())

```

### Metrics

For adding metrics all that is needed is to make a trace with specific prefix. The documentation on how it works is [here](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/struct.MetricsLayer.html#usage)

For adding metrics to axum servers see crates like [tower-otel-http-metrics](https://github.com/francoposa/tower-otel-http-metrics)

## Lints

```sh
cargo clippy --workspace --all-targets
```

and this in your IDE:
```sh
cargo clippy --workspace --all-targets --message-format=json
```

## Pre-commit usage

1. If not installed, install with your package manager, or `pip install --user pre-commit`
2. Run `pre-commit autoupdate` to update the pre-commit config to use the newest template
3. Run `pre-commit install` to install the pre-commit hooks to your local environment

---

# Famedly

**This project is part of the source code of Famedly.**

We think that software for healthcare should be open source, so we publish most
parts of our source code at [github.com/famedly](https://github.com/famedly).

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of
conduct, and the process for submitting pull requests to us.

For licensing information of this project, have a look at the [LICENSE](LICENSE.md)
file within the repository.

If you compile the open source software that we make available to develop your
own mobile, desktop or embeddable application, and cause that application to
connect to our servers for any purposes, you have to agree to our Terms of
Service. In short, if you choose to connect to our servers, certain restrictions
apply as follows:

- You agree not to change the way the open source software connects and
  interacts with our servers
- You agree not to weaken any of the security features of the open source software
- You agree not to use the open source software to gather data
- You agree not to use our servers to store data for purposes other than
  the intended and original functionality of the Software
- You acknowledge that you are solely responsible for any and all updates to
  your software

No license is granted to the Famedly trademark and its associated logos, all of
which will continue to be owned exclusively by Famedly GmbH. Any use of the
Famedly trademark and/or its associated logos is expressly prohibited without
the express prior written consent of Famedly GmbH.

For more
information take a look at [Famedly.com](https://famedly.com) or contact
us by [info@famedly.com](mailto:info@famedly.com?subject=[GitLab]%20More%20Information%20)
