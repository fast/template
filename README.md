# Fast template for developing a new Rust project

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.85][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/${projectName}.svg
[crates-url]: https://crates.io/crates/${projectName}
[docs-badge]: https://img.shields.io/docsrs/${projectName}
[docs-url]: https://docs.rs/${projectName}
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/${projectName}
[license-url]: LICENSE
[actions-badge]: https://github.com/fast/template/workflows/CI/badge.svg
[actions-url]: https://github.com/fast/template/actions?query=workflow%3ACI

Use this repository as a GitHub template to quickly start a new Rust project.

## Getting Started

1. Create a new repository using this template;
2. Clone your repository and run the bootstrap script: `./bootstrap.py`;
3. Follow the prompts, review changes, and commit;
4. Start building your project!

## Minimum Rust version policy

This crate is built against the latest stable release, and its minimum supported rustc version is 1.85.0.

The policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if version 1.0 requires Rust 1.60.0, then version 1.0.z for all values of z will also require Rust 1.60.0 or newer. However, version 1.y for y > 0 may require a newer minimum version of Rust.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
