# cpuinfo-rs title

[![Actions Status](https://github.com/Traverse-Research/cpuinfo-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Traverse-Research/cpuinfo-rs/actions)
[![Latest version](https://img.shields.io/crates/v/cpuinfo-rs.svg?logo=rust)](https://crates.io/crates/cpuinfo-rs)
[![Documentation](https://docs.rs/cpuinfo-rs/badge.svg)](https://docs.rs/cpuinfo-rs)
[![MSRV](https://img.shields.io/badge/rustc-1.74.0+-ab6000.svg)](https://blog.rust-lang.org/2023/11/16/Rust-1.74.0.html)
[![Lines of code](https://tokei.rs/b1/github/Traverse-Research/cpuinfo-rs)](https://github.com/Traverse-Research/cpuinfo-rs)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4%20adopted-ff69b4.svg)](./CODE_OF_CONDUCT.md)

[![Banner](banner.png)](https://traverseresearch.nl)

Thin and slightly opinionated wrapper around [cpuinfo](https://github.com/pytorch/cpuinfo).

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
cpuinfo-rs = "0.1.0"
```

```rust
let info = CpuInfo::new();
dbg!(info.processors());
```
