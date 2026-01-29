# WebX Browser Test Suite

Comprehensive testing framework for WebX browser components.

## Test Organization

```
tests/
├── unit/              # Unit tests for individual components
├── integration/       # Integration tests between modules
├── e2e/              # End-to-end browser tests
├── performance/       # Performance and benchmark tests
├── security/         # Security-focused tests
└── ui/               # User interface tests
```

## Running Tests

### Unit Tests
```bash
# Run all unit tests
cargo test --lib

# Run specific module tests
cargo test features::tabs
cargo test features::security
```

### Integration Tests
```bash
# Run integration tests
cargo test --test integration

# Run specific integration test suites
cargo test --test tab_integration
cargo test --test security_integration
```

### End-to-End Tests
```bash
# Run E2E tests (requires browser instance)
cargo test --test e2e --features "e2e-testing"
```

### Performance Tests
```bash
# Run performance tests
cargo bench --features "testing"

# Run specific performance tests
cargo bench --bench startup_performance
cargo bench --bench rendering_benchmarks
```

## Test Categories

### 1. Core Functionality Tests
- Browser initialization
- Tab management operations
- Navigation and history
- Settings persistence

### 2. Feature Module Tests
- Password manager encryption/decryption
- Ad blocker rule matching
- Download manager functionality
- PDF viewer operations

### 3. Security Tests
- Encryption strength verification
- Privacy protection effectiveness
- Sandbox isolation testing
- Certificate validation

### 4. UI/UX Tests
- Theme switching
- Responsive design
- Accessibility compliance
- User interaction flows

### 5. Performance Tests
- Memory usage patterns
- CPU utilization
- Startup time measurements
- Resource consumption

## Test Infrastructure

### Test Dependencies
```toml
[dev-dependencies]
tempfile = "3.0"
mockall = "0.11"
tokio-test = "0.4"
serial_test = "2.0"
```

### Test Utilities
```rust
// Common test utilities and helpers
pub mod test_utils {
    use tempfile::TempDir;
    
    pub fn create_temp_config() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        // Setup test configuration
        temp_dir
    }
    
    pub fn mock_browser_state() -> BrowserState {
        // Create mock browser state for testing
        BrowserState::new()
    }
}
```

## Writing Tests

### Unit Test Example
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tab_creation() {
        let mut state = BrowserState::new();
        let tab_id = state.add_tab("https://example.com".to_string());
        
        assert!(tab_id > 0);
        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.active_tab_id, Some(tab_id));
    }
    
    #[test]
    fn test_tab_closure() {
        let mut state = BrowserState::new();
        let tab_id = state.add_tab("https://example.com".to_string());
        
        state.remove_tab(tab_id);
        assert_eq!(state.tabs.len(), 0);
        assert_eq!(state.active_tab_id, None);
    }
}
```

### Integration Test Example
```rust
// tests/integration/tab_security_integration.rs
use webx::features::tabs::TabManager;
use webx::features::security::PrivacyProtection;
use webx::core::BrowserState;

#[test]
fn test_tab_isolation_with_privacy_protection() {
    let state = std::sync::Arc::new(std::sync::Mutex::new(BrowserState::new()));
    let tab_manager = TabManager::new(state.clone());
    let privacy = PrivacyProtection::new(None, None).unwrap();
    
    // Create tabs
    let tab1 = tab_manager.create_tab(Some("https://example.com".to_string()));
    let tab2 = tab_manager.create_tab(Some("https://malicious-site.com".to_string()));
    
    // Verify privacy protection applies to malicious tab
    let tab2_url = tab_manager.get_active_tab().unwrap().url;
    assert!(privacy.should_block_url(&tab2_url, &TrackerCategory::Advertising));
    
    // Verify legitimate tab is unaffected
    tab_manager.switch_to_tab(tab1);
    let tab1_url = tab_manager.get_active_tab().unwrap().url;
    assert!(!privacy.should_block_url(&tab1_url, &TrackerCategory::Advertising));
}
```

### Performance Test Example
```rust
// benches/startup_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use webx::ui::BrowserApp;

fn benchmark_browser_startup(c: &mut Criterion) {
    c.bench_function("browser_startup", |b| {
        b.iter(|| {
            let app = BrowserApp::new().expect("Failed to create app");
            black_box(app)
        })
    });
}

criterion_group!(benches, benchmark_browser_startup);
criterion_main!(benches);
```

## Test Coverage Goals

### Current Coverage Targets
- **Core modules**: 90%+ coverage
- **Security features**: 95%+ coverage
- **UI components**: 85%+ coverage
- **Integration points**: 80%+ coverage

### Coverage Measurement
```bash
# Generate coverage report
cargo tarpaulin --out Html

# View coverage summary
cargo tarpaulin --out Xml
```

## Continuous Integration Testing

Tests are automatically run on:
- Every pull request
- Main branch commits
- Scheduled weekly runs
- Release candidate builds

CI Pipeline includes:
- Unit test execution
- Integration test suite
- Security audit scans
- Performance regression checks
- Code quality analysis

## Test Data Management

### Test Fixtures
```bash
# Test data directory structure
tests/data/
├── html/              # Sample HTML documents
├── css/               # Test CSS files
├── js/                # Test JavaScript
├── images/            # Test images
├── pdf/               # Sample PDF documents
└── config/            # Test configuration files
```

### Mock Services
```rust
// Mock HTTP server for testing
pub struct MockHttpServer {
    // Implementation for serving test content
}

// Mock file system for testing
pub struct MockFileSystem {
    // Implementation for file operations
}
```

## Security Testing

### Penetration Testing
- Automated vulnerability scanning
- Manual security assessment
- Third-party security audits
- Bug bounty program considerations

### Compliance Testing
- Privacy regulation compliance
- Data protection requirements
- Security standard adherence
- Audit trail maintenance

## Contributing Tests

### Test Writing Guidelines
1. Follow naming conventions: `test_feature_action_expected`
2. Include both positive and negative test cases
3. Use descriptive assertions with clear failure messages
4. Keep tests isolated and deterministic
5. Clean up test resources properly

### Test Review Process
- All tests require code review
- Performance impact assessment
- Security implication evaluation
- Cross-platform compatibility verification

## Test Environment Setup

### Local Development
```bash
# Install test dependencies
./scripts/install-deps.sh

# Run quick test suite
./scripts/test-quick.sh

# Run full test suite
./scripts/test-all.sh
```

### CI/CD Environment
```yaml
# GitHub Actions workflow example
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install dependencies
        run: ./scripts/install-deps.sh
      - name: Run tests
        run: ./scripts/test-all.sh
      - name: Upload coverage
        uses: codecov/codecov-action@v1
```

## Troubleshooting Tests

Common test issues and solutions:
- **Flaky tests**: Use serial_test crate for ordering
- **Timeout issues**: Increase timeout values for slow operations
- **Resource leaks**: Implement proper cleanup in Drop traits
- **Platform differences**: Use conditional compilation for platform-specific tests