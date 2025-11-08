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

This ensures `hex` remains accessible to users on stable Linux distributions without requiring rustup or third-party repositories. Debian stable's ~2 year release cycle provides predictable, infrequent MSRV updates.

## When to Update MSRV

Update MSRV **only when a new Debian stable release is published** with a newer Rust version.

Check current version at: https://packages.debian.org/stable/rustc

## Update Process

When Debian stable updates:

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
