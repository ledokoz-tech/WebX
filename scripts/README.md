# WebX Browser Development Scripts

## Build Scripts

### Development Build
```bash
#!/bin/bash
# build-dev.sh
cargo build --features "dev-tools"
```

### Release Build
```bash
#!/bin/bash
# build-release.sh
cargo build --release --features "minimal-telemetry"
```

### Cross-platform Build
```bash
#!/bin/bash
# build-cross.sh
# Build for multiple targets
cargo build --target x86_64-pc-windows-gnu
cargo build --target x86_64-unknown-linux-gnu
cargo build --target x86_64-apple-darwin
```

## Testing Scripts

### Run All Tests
```bash
#!/bin/bash
# test-all.sh
cargo test --workspace --features "testing"
```

### Performance Tests
```bash
#!/bin/bash
# test-performance.sh
cargo bench --features "benchmarking"
```

### Security Tests
```bash
#!/bin/bash
# test-security.sh
cargo test security_tests --features "security-audit"
```

## Deployment Scripts

### Package for Distribution
```bash
#!/bin/bash
# package.sh
# Create distribution packages
VERSION=$(grep '^version =' Cargo.toml | cut -d '"' -f 2)
echo "Packaging WebX v$VERSION"

# Create directories
mkdir -p dist/webx-$VERSION/{windows,linux,macos}

# Build for each platform
cross build --target x86_64-pc-windows-gnu --release
cross build --target x86_64-unknown-linux-gnu --release
cross build --target x86_64-apple-darwin --release

# Copy binaries
cp target/x86_64-pc-windows-gnu/release/webx.exe dist/webx-$VERSION/windows/
cp target/x86_64-unknown-linux-gnu/release/webx dist/webx-$VERSION/linux/
cp target/x86_64-apple-darwin/release/webx dist/webx-$VERSION/macos/

# Create archives
cd dist
tar -czf webx-$VERSION-windows.tar.gz webx-$VERSION/windows
tar -czf webx-$VERSION-linux.tar.gz webx-$VERSION/linux
tar -czf webx-$VERSION-macos.tar.gz webx-$VERSION/macos
```

### Install Dependencies
```bash
#!/bin/bash
# install-deps.sh
# Install required development dependencies

echo "Installing WebX development dependencies..."

# Rust tools
rustup component add clippy
rustup component add rustfmt
rustup component add miri

# System dependencies (Ubuntu/Debian)
if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y \
        libwebkit2gtk-4.0-dev \
        build-essential \
        curl \
        wget \
        pkg-config \
        libssl-dev \
        libgtk-3-dev
fi

# System dependencies (macOS)
if command -v brew &> /dev/null; then
    brew install webkit2gtk
fi

# Install cargo tools
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-bloat
cargo install cross

echo "Dependencies installed successfully!"
```

## Development Tools

### Code Quality Checks
```bash
#!/bin/bash
# quality-check.sh
echo "Running code quality checks..."

# Format code
cargo fmt --all -- --check

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit

# Check for unused dependencies
cargo machete

echo "Quality checks completed!"
```

### Watch Mode Development
```bash
#!/bin/bash
# watch-dev.sh
# Development with auto-rebuild
cargo watch -x "run --features dev-mode"
```

### Generate Documentation
```bash
#!/bin/bash
# gen-docs.sh
# Generate and open documentation
cargo doc --no-deps --open
```

## CI/CD Helper Scripts

### Pre-commit Hook
```bash
#!/bin/bash
# pre-commit.sh
# Run before each commit

echo "Running pre-commit checks..."

# Run quality checks
./scripts/quality-check.sh

# Run fast tests
cargo test --lib

if [ $? -ne 0 ]; then
    echo "Pre-commit checks failed!"
    exit 1
fi

echo "Pre-commit checks passed!"
```

### Release Preparation
```bash
#!/bin/bash
# prepare-release.sh
# Prepare for new release

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: ./prepare-release.sh <version>"
    exit 1
fi

echo "Preparing release v$VERSION..."

# Update version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update version in README
sed -i "s/WebX v[0-9]*\.[0-9]*\.[0-9]*/WebX v$VERSION/" README.md

# Generate changelog
echo "## [v$VERSION] - $(date +%Y-%m-%d)" > temp_changelog.md
echo "" >> temp_changelog.md
echo "### Added" >> temp_changelog.md
echo "- [Features added in this version]" >> temp_changelog.md
echo "" >> temp_changelog.md
echo "### Changed" >> temp_changelog.md
echo "- [Changes made in this version]" >> temp_changelog.md
echo "" >> temp_changelog.md
echo "### Fixed" >> temp_changelog.md
echo "- [Bugs fixed in this version]" >> temp_changelog.md

# Prepend to CHANGELOG.md
cat temp_changelog.md CHANGELOG.md > temp && mv temp CHANGELOG.md
rm temp_changelog.md

echo "Release preparation completed!"
echo "Next steps:"
echo "1. Review CHANGELOG.md"
echo "2. Run tests: ./scripts/test-all.sh"
echo "3. Commit changes"
echo "4. Create git tag: git tag v$VERSION"
echo "5. Push: git push origin main v$VERSION"