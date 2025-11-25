# Release Process

This document describes how to create a new release of miam.

## Creating a New Release

miam uses GitHub Actions for automated releases. When you push a new version tag, the workflow automatically:

1. Builds binaries for multiple platforms (Linux x86_64, Linux ARM64, macOS x86_64, macOS ARM64)
2. Uploads the binaries to a GitHub Release
3. Generates release notes with installation instructions

### Steps to Create a Release

1. **Update the version in `Cargo.toml`**

   ```toml
   [package]
   version = "0.2.0"  # Update this
   ```

2. **Commit the version change**

   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   ```

3. **Create and push a git tag**

   ```bash
   git tag v0.2.0
   git push origin master
   git push origin v0.2.0
   ```

4. **Wait for GitHub Actions to complete**

   The workflow will automatically build binaries for all platforms and create a GitHub Release.

   You can monitor the progress at:
   https://github.com/saravenpi/miam/actions

5. **Verify the Release**

   Once the workflow completes, check:
   - https://github.com/saravenpi/miam/releases

   The release should include:
   - Release notes with installation instructions
   - Four binary assets:
     - `miam-linux-x86_64`
     - `miam-linux-aarch64`
     - `miam-macos-x86_64`
     - `miam-macos-aarch64`

## Version Numbering

miam follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (0.x.0): Breaking changes
- **MINOR** (x.1.0): New features, backward compatible
- **PATCH** (x.x.1): Bug fixes, backward compatible

## Testing the Upgrade Command

After creating a release, users can upgrade using:

```bash
miam upgrade
```

This will:
- Check GitHub for the latest release
- Download the appropriate binary for their platform
- Replace the current binary
- Show a success message

## Manual Testing Before Release

Before creating a release, ensure:

1. **Code compiles**
   ```bash
   cargo build --release
   ```

2. **Tests pass** (when added)
   ```bash
   cargo test
   ```

3. **Binary works**
   ```bash
   ./target/release/miam --version
   ./target/release/miam --help
   ```

4. **TUI launches correctly**
   ```bash
   ./target/release/miam
   ```

## Rollback

If a release has issues:

1. Delete the tag locally and remotely:
   ```bash
   git tag -d v0.2.0
   git push origin :refs/tags/v0.2.0
   ```

2. Delete the GitHub Release through the web interface

3. Fix the issues and create a new release with an incremented version
