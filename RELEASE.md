# Release Process

strop follows [Semantic Versioning](https://semver.org/).

## Version numbering

Given version `MAJOR.MINOR.PATCH`:

- **MAJOR** — breaking changes to the CLI interface, config format, or protocol
- **MINOR** — new features, new commands, new device support
- **PATCH** — bug fixes, performance improvements, documentation

Before 1.0.0 (`0.x.y`):
- **MINOR** (`0.1.0` → `0.2.0`) — breaking changes or new features
- **PATCH** (`0.1.0` → `0.1.1`) — bug fixes only

Pre-release tags: `v0.2.0-alpha.1`, `v0.2.0-rc.1`

## Release steps

```sh
# 1. Update the version in Cargo.toml (root + any changed workspace crates)
#    Example: 0.1.0 → 0.1.1
$EDITOR Cargo.toml
cargo build --release

# 2. Commit and push
git commit -am "release: v0.1.1"
git push

# 3. Tag and push tags (triggers the release workflow)
git tag v0.1.1
git push --tags
```

The CI release workflow (`.github/workflows/ci.yml`) will:
1. Build release binaries for macOS (x86_64 + arm64), Linux (x86_64), Windows (x86_64)
2. Create a GitHub Release with the binaries attached
3. Generate release notes from commits since the last tag

## Pre-release / release candidate

```sh
git tag v0.2.0-rc.1
git push --tags
```

This creates a GitHub Release marked as "Pre-release". Use `v0.2.0-rc.1`,
`v0.2.0-rc.2`, etc., then the final `v0.2.0` when ready.

## Version consistency

The root package version (`strop`) is the canonical release version.
Workspace crate versions (`strop-proto`, `strop-hid`, etc.) are independent
and bumped as needed — they are library crates that could be published
separately. For a strop binary release, only the root version matters.
