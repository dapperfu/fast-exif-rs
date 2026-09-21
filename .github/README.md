# GitHub Actions

Workflows match the current **pure Rust** crate (`fast-exif-reader`) and CLI (`exiftool-rs`). There is no Python/maturin package.

## Workflows

### CI (`ci.yml`)

Runs on pushes to `main`/`develop` and pull requests to `main`.

- Test matrix: Ubuntu, Windows, macOS (`cargo test` for the library and CLI)
- Lint: `cargo fmt --check` and `clippy -D warnings`
- Security: `rustsec/audit-check`

### Integration Test (`integration-test.yml`)

Release-mode build plus `exiftool-rs --help` smoke test.

### Build and Release (`build-and-release.yml`)

Triggers on `v*` tags (and manual dispatch). Builds `exiftool-rs` binaries for Linux, Windows, and macOS, then attaches them to a GitHub Release.

```bash
git tag -a v0.11.2 -m "Release v0.11.2"
git push origin v0.11.2
```
