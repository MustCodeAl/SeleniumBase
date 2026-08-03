# Docker Guide

Run SeleniumBase for Rust tests inside a container for reproducible CI/CD builds.

## Build the image

From the `rust-port` directory:

```bash
docker build -t seleniumbase-rs .
```

From the repository root:

```bash
docker build -f rust-port/Dockerfile -t seleniumbase-rs ./rust-port
```

## Run the CLI

The runtime image contains the compiled `sbase` binary and Chromium. Run a
headless command, for example:

```bash
docker run --rm seleniumbase-rs sbase --help
```

## Docker Compose

```yaml
services:
  tests:
    build: .
    environment:
      - SBASE_HEADLESS=true
    command: ["sbase", "--help"]
```

Run with Compose:

```bash
docker compose up --build
```

## CI/CD

See `.github/workflows/` for ready-to-use GitHub Actions jobs:

- `build.yml` — build and test on every push.
- `clippy.yml` — lint check.
- `examples.yml` — run example suite.
- `docker-rs.yml` — build and push the Rust Docker image (repository root only).

## Headless in Docker

Always run browsers in headless mode inside containers; the image already adds
`--no-sandbox` for Chromium when headless mode is enabled:

```rust
let config = BrowserConfig::default().with_headless(true);
```
