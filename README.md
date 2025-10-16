# cctp-bridge

[![Crates.io](https://img.shields.io/crates/v/cctp-bridge.svg)](https://crates.io/crates/cctp-bridge)
[![Docs.rs](https://docs.rs/cctp-bridge/badge.svg)](https://docs.rs/cctp-bridge)
[![CI](https://github.com/CarteraMesh/cctp-bridge/workflows/test/badge.svg)](https://github.com/CarteraMesh/cctp-bridge/actions)
[![Cov](https://codecov.io/github/CarteraMesh/cctp-bridge/graph/badge.svg?token=dILa1k9tlW)](https://codecov.io/github/CarteraMesh/cctp-bridge)

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install cctp-bridge`


## Development

### Prerequisites

- **Rust Nightly**: Required for code formatting with advanced features
  ```bash
  rustup install nightly
  ```

### Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/CarteraMesh/cctp-bridge.git
   cd cctp-bridge
   ```

2. **Build and test**
   ```bash
   # Build the project
   cargo build

   # Run tests (requires valid Fireblocks credentials in .env)
   cargo test

   # Format code (requires nightly)
   cargo +nightly fmt --all
   ```

### Code Formatting

This project uses advanced Rust formatting features that require nightly:

```bash
# Format all code
cargo +nightly fmt --all

# Check formatting
cargo +nightly fmt --all -- --check
```

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
