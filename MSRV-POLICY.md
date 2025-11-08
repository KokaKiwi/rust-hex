# Minimum Supported Rust Version (MSRV) Policy

## Policy

The `hex` crate's MSRV **must not exceed the Rust version in the current stable Debian release**.

## Current MSRV

**Rust 1.85** (Debian Trixie stable as of 2025)

Specified in `Cargo.toml`:
```toml
[package]
rust-version = "1.85"
```

## Rationale

This ensures `hex` remains accessible to users on stable Linux distributions without requiring rustup or third-party repositories, while allowing the crate to adopt useful Rust features as they become available within that constraint.

## When to Update MSRV

MSRV may be updated when:
- **A useful Rust feature becomes available** that would benefit the crate
- **The new version does not exceed** the Rust version in current stable Debian

MSRV updates **may occur in minor releases** (not just major releases).

Check current Debian stable Rust version at: https://packages.debian.org/stable/rustc

## Update Process

When updating MSRV:

1. **Update `Cargo.toml`**:
   ```toml
   rust-version = "X.YY"
   ```

2. **Update documentation**: This file, README.md (if applicable)

3. **Update CI**: `.github/workflows/build.yml` MSRV test matrix

4. **Test all feature combinations**:
   ```bash
   rustup toolchain install X.YY
   cargo +X.YY test --no-default-features
   cargo +X.YY test --no-default-features --features alloc
   cargo +X.YY test --all-features
   cargo +X.YY build --no-default-features --target thumbv6m-none-eabi
   ```
