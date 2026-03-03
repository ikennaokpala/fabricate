# Claude Code Configuration — Fabricate

## Overview

Fabricate is a FactoryBot-inspired test data factory for Rust + sqlx. It generates realistic, persona-aware test data for the RideMate backend via CLI or library API.

## Build & Test

```bash
cargo test        # Run all tests (17 unit + integration)
cargo build       # Build CLI binary at target/debug/fabricate
```

- ALWAYS run `cargo test` after making code changes
- ALWAYS verify `cargo build` succeeds before committing

## Testing Rules

### Mocking Policy (STRICT)

- NEVER mock, stub, or spy on internal code — our own factories, builders, contexts, personas, or any code we wrote
- NEVER use test doubles (mocks, stubs, fakes, spies) for internal dependencies
- ONLY mock external APIs and third-party libraries that we did not write (e.g., external HTTP services, cloud storage providers)
- All internal code paths MUST be exercised for real in tests
- Database connections and internal HTTP clients are internal — never mock them

### Test Data Seeding Strategy

When seeding test data for a separated frontend/backend architecture:

1. **Priority 1 — Local backend + CLI seeding**: Run the backend locally and use the fabricate CLI binary to seed data directly via the backend's test endpoints. This is the default and preferred approach.
2. **Priority 2 — API fallback**: Only if the local backend cannot be instantiated (e.g., CI environment without Docker, missing database), fall back to HTTP API-based seeding methods.
3. NEVER jump straight to API seeding when a local backend is available
4. The fabricate CLI binary is the canonical seeding tool — all seeding flows go through it

## Architecture

- `src/context.rs` — Shared context (HTTP client, DB pool, sequences, overrides)
- `src/builder.rs` — Generic factory builder with trait application
- `src/ridemate/` — RideMate-specific factories (user, ride, driver, payment, trip, etc.)
- `src/ridemate/personas.rs` — Pre-built persona bundles and `seed_via_api()` for backend seeding
- `src/cli.rs` — CLI entry point (`health`, `seed` commands)

## URL Construction

Backend test endpoints are mounted at `/api/v1/test/__test__/*`. The `test_post()` and `test_delete()` methods in `context.rs` prepend `/api/v1/test` to all paths automatically. Factory persist methods should use bare paths like `/__test__/users`.

## Security Rules

- NEVER hardcode API keys, secrets, or credentials in source files
- NEVER commit .env files or any file containing secrets
- Test API keys (`test-key`) are only for local/staging test endpoints gated behind `#[cfg(feature = "test-endpoints")]`
