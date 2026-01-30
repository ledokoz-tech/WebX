# GitHub Actions CI/CD Documentation

## Overview

WebX uses GitHub Actions for continuous integration, automated building, and release management. The system automatically builds for multiple platforms and creates releases based on commit count.

## Workflows

### 1. Build and Release (`build-release.yml`)

**Trigger**: Push to `main` branch

**Function**: Builds WebX for all supported platforms and creates GitHub releases.

**Platforms Supported**:
- **Windows**: EXE and MSI installer
- **Linux**: DEB package, AppImage, and standalone binary
- **macOS**: DMG installer and universal binary (Intel + Apple Silicon)
- **Android**: APK (experimental)

**Versioning System**:
Uses commit-count based versioning:
- 1 commit = `0.0.001`
- 10 commits = `0.0.01`
- 100 commits = `0.0.1`
- 1,000 commits = `0.1.0`
- 10,000 commits = `1.0.0`

### 2. CI Tests (`ci.yml`)

**Trigger**: Pull requests and pushes to `develop`/feature branches

**Function**: Runs tests, formatting checks, and builds across multiple platforms.

**Matrix Testing**:
- Ubuntu, Windows, and macOS
- Different target architectures
- Code quality checks (clippy, fmt)

### 3. Security Audit (`security.yml`)

**Trigger**: Weekly and dependency file changes

**Function**: Runs security audits and dependency checks.

## Version Calculation

The version is calculated automatically using the commit count:

```bash
# From scripts/calculate-version.sh
COMMIT_COUNT=$(git rev-list --count HEAD)
MAJOR=$((COMMIT_COUNT / 10000))
MINOR=$((COMMIT_COUNT % 10000 / 1000))
PATCH=$((COMMIT_COUNT % 1000 / 100))
BUILD=$((COMMIT_COUNT % 100))
```

## Artifact Naming Convention

All artifacts follow this pattern:
```
webx-{version}-{platform}-{architecture}.{extension}
```

Examples:
- `webx-0.1.2-windows-x64.exe`
- `webx-0.1.2-linux-amd64.deb`
- `webx-0.1.2-macos-universal.dmg`

## Release Process

1. **Push to main** triggers the build workflow
2. **Version calculation** happens automatically
3. **Multi-platform builds** run in parallel
4. **Artifacts are collected** and packaged
5. **GitHub Release** is created with all artifacts
6. **Release notes** are auto-generated with version info

## Required Secrets

No special secrets are required for basic functionality. The workflow uses:
- `GITHUB_TOKEN` (automatically provided)
- Standard runner permissions

## Customization

### Adding New Platforms
1. Add new job in `build-release.yml`
2. Configure platform-specific build steps
3. Update artifact naming convention
4. Add platform to release documentation

### Modifying Version Scheme
Edit `scripts/calculate-version.sh` and `scripts/calculate-version.ps1` to change the version calculation logic.

### Adding Build Steps
Modify the appropriate job sections in `build-release.yml` to add:
- Additional testing
- Code signing
- Custom packaging
- Post-build validation

## Troubleshooting

### Common Issues

**Build Failures**:
- Check platform-specific dependencies
- Verify Rust toolchain versions
- Review build logs for specific errors

**Version Calculation Problems**:
- Ensure full git history is fetched (`fetch-depth: 0`)
- Check script permissions on Unix systems
- Validate PowerShell execution policy on Windows

**Artifact Upload Issues**:
- Verify artifact paths exist
- Check file permissions
- Ensure sufficient disk space

### Debugging Tips

1. **Enable verbose logging** by adding `set -x` to shell scripts
2. **Use debug builds** during development
3. **Test locally** before pushing changes
4. **Check GitHub Actions logs** for detailed error information

## Best Practices

### For Contributors
- Run `cargo fmt` and `cargo clippy` locally before pushing
- Test changes on multiple platforms when possible
- Keep pull requests focused and well-documented

### For Maintainers
- Monitor build failures and address promptly
- Regular security audits
- Keep dependencies updated
- Review and test release candidates

## Future Enhancements

Planned improvements:
- Code signing for Windows and macOS
- Automatic deployment to package repositories
- Container image builds
- Performance benchmarking integration
- Automated changelog generation