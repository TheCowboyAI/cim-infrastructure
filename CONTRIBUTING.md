# Contributing to CIM Infrastructure

Thank you for your interest in contributing to CIM Infrastructure! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- **Nix with Flakes** (recommended) OR Rust 1.70+
- **NATS Server** (for integration testing)
- **Git** for version control

### Getting Started with Nix (Recommended)

```bash
# Clone the repository
git clone https://github.com/thecowboyai/cim-infrastructure.git
cd cim-infrastructure

# Enter development shell
nix develop

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace
```

### Getting Started without Nix

```bash
# Clone the repository
git clone https://github.com/thecowboyai/cim-infrastructure.git
cd cim-infrastructure

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install NATS server
# See: https://docs.nats.io/running-a-nats-service/introduction/installation

# Build and test
cargo build --workspace
cargo test --workspace
```

## CI/CD Pipeline

We use GitHub Actions for continuous integration and deployment:

### Continuous Integration (`.github/workflows/ci.yml`)

Runs on every push and pull request:

- **Test Suite**: Runs `cargo test` with multiple feature combinations
  - No default features
  - All features (including optional `policy`)
- **Clippy**: Linting with `cargo clippy --all-targets --all-features`
- **Rustfmt**: Code formatting checks with `cargo fmt --check`
- **Build**: Builds in both debug and release modes
- **Examples**: Validates all examples run successfully
- **Security Audit**: Runs `cargo audit` for dependency vulnerabilities
- **Code Coverage**: Generates coverage reports with `cargo-tarpaulin`

### NATS Integration Tests (`.github/workflows/nats-integration.yml`)

Runs on push, pull requests, and daily:

- **Single Node**: Tests against standalone NATS server with JetStream
- **Cluster Simulation**: Tests against 3-node NATS cluster

### Release Automation (`.github/workflows/release.yml`)

Triggered on version tags (`v*`):

- **GitHub Release**: Automatic changelog generation and release creation
- **Build Artifacts**: Multi-platform binary builds (Linux x86_64/aarch64, macOS x86_64/aarch64)
- **Crates.io Publishing**: Publishes to crates.io (requires `CARGO_REGISTRY_TOKEN`)
- **Docker Images**: Builds and pushes to GitHub Container Registry
- **Documentation**: Deploys rustdoc to GitHub Pages

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Changes

Follow these principles:

- **Event Sourcing**: All state changes as immutable events
- **Domain-Driven Design**: Clear bounded contexts and aggregates
- **Framework Independence**: No coupling to specific frameworks
- **Type Safety**: Strongly typed value objects with validation

### 3. Write Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific module
cargo test -p cim-domain-infrastructure

# Run with all features
cargo test --workspace --all-features

# Run NATS integration tests (requires NATS server)
nats-server -js &
cargo test -p cim-infrastructure-nats --test integration_test -- --ignored
```

### 4. Check Code Quality

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

### 5. Run Examples

```bash
# Topology visualization
cargo run --example topology_visualization

# Network provisioning saga
cargo run --example network_provisioning_saga
```

### 6. Commit Changes

Follow conventional commits:

```bash
git add .
git commit -m "feat: add new infrastructure event type"
```

Commit message prefixes:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Test additions/changes
- `refactor:` Code refactoring
- `chore:` Maintenance tasks

### 7. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Testing Guidelines

### Unit Tests

- Place tests in the same file as the code using `#[cfg(test)]` modules
- Test all public APIs
- Test error conditions
- Test edge cases

### Integration Tests

- Place in `tests/` directory
- Test interactions between modules
- For NATS tests, mark with `#[ignore]` and require real NATS server

### Documentation Tests

- Include examples in doc comments
- Ensure examples compile and run

## Optional Features

The workspace supports optional features:

```toml
[features]
default = []
policy = ["dep:cim-domain-policy"]
```

Test with features:

```bash
# Default features only
cargo test --workspace

# All features
cargo test --workspace --all-features

# Specific feature
cargo test --workspace --features policy
```

## Architecture Compliance

All contributions must follow CIM best practices:

### Event Sourcing Patterns

- ✅ Command handlers with validation
- ✅ Event application for state changes
- ✅ Correlation and causation tracking
- ✅ Event replay capability
- ✅ Read model projections
- ❌ NO CRUD operations

### NATS-First Communication

- ✅ All communication via NATS messaging
- ✅ Subject hierarchy: `infrastructure.{aggregate}.{operation}`
- ✅ JetStream for persistent events

### Category Theory Integration

- ✅ Use functors for domain mapping (see `cim_graph_integration.rs`)
- ✅ Leverage Kan extensions for universal properties
- ✅ Compose existing CIM modules, don't rebuild

### UUID v7

- ✅ Always use `Uuid::now_v7()` for time-ordered identifiers
- ❌ Never use v4 or v5

## Release Process

1. Update version in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Update `STATUS.md` if needed
4. Commit changes: `git commit -m "chore: bump version to X.Y.Z"`
5. Create and push tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
6. GitHub Actions will automatically:
   - Create GitHub release
   - Build artifacts
   - Publish to crates.io (if configured)
   - Build and push Docker image
   - Deploy documentation

## Getting Help

- **Issues**: Open an issue on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check rustdoc at https://thecowboyai.github.io/cim-infrastructure/

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the project's architectural principles

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).
