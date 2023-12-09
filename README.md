# cargo-semver-bump

[![Docs](https://img.shields.io/docsrs/cargo-semver-bump)](https://docs.rs/cargo-semver-bump)
[![GitHub workflow status](https://github.com/markhaehnel/cargo-semver-bump/actions/workflows/ci.yaml/badge.svg)](https://github.com/markhaehnel/cargo-semver-bump/actions/workflows/ci.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/cargo-semver-bump)](https://crates.io/crates/cargo-semver-bump)
[![Crates.io Downloads](https://img.shields.io/crates/d/cargo-semver-bump)](https://crates.io/crates/cargo-semver-bump)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

> 🚧 **WORK IN PROGRESS** 🚧
>
> This tool is still in early ydevelopment and not ready for production use.
> Breaking changes may occur at any time.

cargo-semver-bump is a tool to automatically bump the version of your Rust crate based on the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification and breaking changes detected by [cargo-semver-checks](https://crates.io/crates/cargo-semver-checks).

## Installation

### From crates.io

```bash
cargo install cargo-semver-bump --locked
```

## Usage

Just run the following command in your project directory:

```bash
cargo semver-bump
```

More options are available:

```bash
$ cargo semver-bump --help

Usage: cargo semver-bump [OPTIONS]

Options:
  -d, --dry-run      Run without making any changes
  -p, --path <PATH>  Path to project [default: ./]
  -v, --verbose      Enable verbose output
  -h, --help         Print help
  -V, --version      Print version
```

## Limitations

- Only works with Git repositories
- Only works with Cargo.toml files in the root of the repository
- [Limitations of cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks?tab=readme-ov-file#does-cargo-semver-checks-have-false-positives)

## Contributing

See the [contributing guidelines](./CONTRIBUTING.md) for more information.

## License

This code is licensed under either of

- [MIT License](./LICENSE-MIT)
- [Apache-2.0 License](./LICENSE-APACHE)

at your option.
