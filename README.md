# fabricate

[![Crates.io](https://img.shields.io/crates/v/fabricate.svg)](https://crates.io/crates/fabricate)
[![docs.rs](https://img.shields.io/docsrs/fabricate)](https://docs.rs/fabricate)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**FactoryBot-inspired test data factory for Rust + sqlx**

fabricate brings the ergonomics of Ruby's [FactoryBot](https://github.com/thoughtbot/factory_bot) to Rust. Define factories once, then build in-memory structs, persist directly to Postgres via sqlx, or seed through your HTTP API — all with composable traits, auto-incrementing sequences, and field-level overrides.

---

## Table of Contents

- [Why fabricate?](#why-fabricate)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
  - [Factory and BuildableFactory](#factory-and-buildablefactory)
  - [FactoryBuilder (Fluent API)](#factorybuilder-fluent-api)
  - [Traits (Composable Modifications)](#traits-composable-modifications)
  - [Sequences (Unique Value Generation)](#sequences-unique-value-generation)
  - [Field Overrides](#field-overrides)
- [Persistence Modes](#persistence-modes)
  - [Build (In-Memory)](#build-in-memory)
  - [Create via Postgres](#create-via-postgres)
  - [Create via HTTP API](#create-via-http-api)
- [Personas (Scenario Bundles)](#personas-scenario-bundles)
- [Defining Your Own Factory](#defining-your-own-factory)
- [Built-in Factories Reference](#built-in-factories-reference)
- [CLI Tool](#cli-tool)
- [Architecture Overview](#architecture-overview)
- [Cargo Features](#cargo-features)
- [Error Handling](#error-handling)
- [TypeScript Client](#typescript-client)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## Why fabricate?

Rust's test ecosystem has excellent assertion libraries, mocking tools, and property-based testing — but no standard answer for **test data factories**. Setting up realistic, interconnected test entities means writing dozens of builder functions by hand, managing foreign key relationships manually, and duplicating setup logic across test files.

fabricate solves this with:

- **Declarative factories** — define entity defaults once, reuse everywhere
- **Composable traits** — named modifications that stack (`"verified"`, `"premium"`, `"suspended"`)
- **Auto-incrementing sequences** — unique emails, phones, names, and license plates out of the box
- **Field-level overrides** — override any field inline with `.set("email", json!("custom@test.com"))`
- **Three persistence modes** — build in-memory, persist to Postgres via sqlx, or seed via HTTP API
- **Persona bundles** — pre-composed multi-entity scenarios (rider + wallet + payment, booking flow, etc.)
- **CLI tool** — seed and reset test data from the command line

### Comparison

| Feature | Hand-written builders | `fake` / `faker` | **fabricate** |
|---|---|---|---|
| Reusable entity definitions | Manual per-test | No structure | **Factories with defaults** |
| Named variants (verified, suspended) | If-else chains | N/A | **Composable traits** |
| Unique values (emails, phones) | Manual counters | Random (collisions) | **Deterministic sequences** |
| Database persistence | Custom per-entity | N/A | **Built-in (sqlx + HTTP)** |
| Multi-entity scenarios | Large setup blocks | N/A | **Personas** |
| Relationships / foreign keys | Manual wiring | N/A | **Associations** |

---

## Quick Start

Add fabricate to your `Cargo.toml`:

```toml
[dev-dependencies]
fabricate = "0.1"
serde_json = "1"
```

To use direct Postgres persistence (enabled by default):

```toml
[dev-dependencies]
fabricate = { version = "0.1", features = ["postgres"] }
```

Build your first entity:

```rust
use fabricate::{FactoryBuilder, FactoryContext, Sequence};
use fabricate::ridemate::user::UserFactory;
use serde_json::json;

// Create a build-only context (no database, no HTTP)
let mut ctx = FactoryContext {
    sequences: Sequence::new(),
    pool: None,
    http_client: None,
    base_url: None,
    test_key: "test-key".to_string(),
    overrides: std::collections::HashMap::new(),
};

// Build a default user (in-memory only)
let user = FactoryBuilder::new(UserFactory::new())
    .build(&mut ctx)
    .unwrap();

assert!(user.email.starts_with("test_user_"));
assert_eq!(user.user_type, "passenger");

// Build a verified driver with a custom email
let driver = FactoryBuilder::new(UserFactory::new())
    .with_trait("verified")
    .with_trait("driver")
    .set("email", json!("driver@example.com"))
    .build(&mut ctx)
    .unwrap();

assert_eq!(driver.email, "driver@example.com");
assert_eq!(driver.user_type, "driver");
assert!(driver.is_email_verified);
assert!(driver.is_phone_verified);
```

---

## Core Concepts

### Factory and BuildableFactory

fabricate has two core traits for defining factories:

**`Factory`** — the high-level async trait with `build` and `create` methods:

```rust
#[async_trait]
pub trait Factory: Send + Sync {
    type Output: Send;

    /// Build an in-memory instance (no persistence).
    fn build(&self, ctx: &mut FactoryContext) -> Self::Output;

    /// Persist to database and return the created record.
    async fn create(&self, ctx: &mut FactoryContext) -> Result<Self::Output>;

    /// Create multiple entities at once.
    async fn create_list(&self, ctx: &mut FactoryContext, count: usize) -> Result<Vec<Self::Output>>;
}
```

**`BuildableFactory<T>`** — the lower-level trait used by `FactoryBuilder`, giving you control over base construction, trait registries, field overrides, and persistence:

```rust
pub trait BuildableFactory<T>: Send + Sync {
    fn build_base(&self, ctx: &mut FactoryContext) -> T;
    fn trait_registry(&self) -> &TraitRegistry<T>;
    fn apply_overrides(&self, entity: &mut T, overrides: &[(String, serde_json::Value)]);
    async fn persist(&self, entity: T, ctx: &mut FactoryContext) -> Result<T>;
}
```

Most factories implement `BuildableFactory<T>` and are used through `FactoryBuilder`.

### FactoryBuilder (Fluent API)

`FactoryBuilder` provides the chainable API for constructing entities:

```rust
let user = FactoryBuilder::new(UserFactory::new())
    .with_trait("verified")       // Apply a named trait
    .with_trait("premium")        // Stack multiple traits
    .set("email", json!("vip@example.com"))  // Override a field
    .build(&mut ctx)              // Build in-memory
    .unwrap();
```

Methods:

| Method | Description |
|---|---|
| `FactoryBuilder::new(factory)` | Wrap a factory in the fluent builder |
| `.with_trait("name")` | Apply a named trait (composable, order matters) |
| `.set("field", json!(value))` | Override a specific field value |
| `.build(&mut ctx)` | Build in-memory (synchronous, returns `Result<T>`) |
| `.create(&mut ctx).await` | Build + persist to database or HTTP API (async) |

### Traits (Composable Modifications)

Traits are named modifications that alter specific fields on an entity. They are composable — apply multiple traits and they stack in order, with later traits overriding earlier ones for the same fields.

```rust
// Single trait
let verified_user = FactoryBuilder::new(UserFactory::new())
    .with_trait("verified")
    .build(&mut ctx)
    .unwrap();

// Stacked traits — "driver" overrides user_type set by defaults
let verified_driver = FactoryBuilder::new(UserFactory::new())
    .with_trait("verified")
    .with_trait("driver")
    .build(&mut ctx)
    .unwrap();

assert!(verified_driver.is_email_verified);
assert_eq!(verified_driver.user_type, "driver");
```

To define a trait for your own factory, implement `FactoryTrait<T>`:

```rust
pub trait FactoryTrait<T>: Send + Sync {
    fn name(&self) -> &str;
    fn apply(&self, entity: &mut T);
}
```

### Sequences (Unique Value Generation)

Every `FactoryContext` includes a `Sequence` generator that produces unique, deterministic values. Named sequences auto-increment independently starting from 1.

```rust
// Raw sequence counter
let n = ctx.sequence("invoice");  // 1, 2, 3, ...

// Built-in helpers
let email = ctx.email("rider");   // "test_rider_1@test.ridemate.app"
let phone = ctx.phone();          // "+15550000001"
let name  = ctx.full_name();      // "Alice Smith"
```

Built-in sequence helpers:

| Helper | Example Output | Pattern |
|---|---|---|
| `ctx.email("prefix")` | `test_prefix_1@test.ridemate.app` | Unique per call |
| `ctx.phone()` | `+15550000001` | 555 area code, zero-padded |
| `ctx.full_name()` | `Alice Smith` | Cycles through 10 first/last names |
| `sequences.plate()` | `AA0001` | Letter pairs + zero-padded number |
| `ctx.sequence("name")` | `1`, `2`, `3`... | Raw counter for any named sequence |

### Field Overrides

Override any field on any factory using `.set()` with a JSON value:

```rust
let user = FactoryBuilder::new(UserFactory::new())
    .set("email", json!("custom@test.com"))
    .set("full_name", json!("Custom Name"))
    .build(&mut ctx)
    .unwrap();

assert_eq!(user.email, "custom@test.com");
assert_eq!(user.full_name, "Custom Name");
```

Overrides are applied **after** traits, so they always win. Each factory defines which fields are overridable in its `apply_overrides` implementation.

---

## Persistence Modes

### Build (In-Memory)

The simplest mode — build entities as plain Rust structs with no side effects:

```rust
let mut ctx = FactoryContext {
    sequences: Sequence::new(),
    pool: None,
    http_client: None,
    base_url: None,
    test_key: "test-key".to_string(),
    overrides: std::collections::HashMap::new(),
};

let user = FactoryBuilder::new(UserFactory::new())
    .build(&mut ctx)
    .unwrap();
// user is a TestUser struct — no DB, no network
```

### Create via Postgres

Persist entities directly to your Postgres database using sqlx. Requires the `postgres` feature (enabled by default).

```rust
let pool = sqlx::PgPool::connect("postgres://localhost/myapp_test").await?;
let mut ctx = FactoryContext::database(pool);

let user = FactoryBuilder::new(UserFactory::new())
    .with_trait("verified")
    .create(&mut ctx)  // INSERT INTO users ...
    .await?;
// user.id is now a real database UUID
```

Each factory's `persist` method contains the actual SQL. For example, `UserFactory` executes:

```sql
INSERT INTO users (id, email, phone_number, password_hash, full_name, ...)
VALUES ($1, $2, $3, $4, $5, ...)
ON CONFLICT (email) DO UPDATE SET updated_at = $12
RETURNING id
```

### Create via HTTP API

Seed data through your running backend's test API endpoints. Factories POST to `/__test__/` routes with `X-Test-Key` authentication:

```rust
let mut ctx = FactoryContext::http("http://localhost:8080")
    .with_test_key("my-secret-test-key");

let user = FactoryBuilder::new(UserFactory::new())
    .with_trait("verified")
    .create(&mut ctx)  // POST http://localhost:8080/__test__/users
    .await?;
```

HTTP mode sends JSON payloads and expects JSON responses. The `X-Test-Key` header authenticates requests so your test endpoints can reject unauthorized access in production.

---

## Personas (Scenario Bundles)

Personas compose multiple factories into realistic multi-entity scenarios. Instead of manually creating a user, wallet, payment method, driver profile, and ride, call a single persona method.

```rust
use fabricate::ridemate::personas::Personas;

let mut ctx = FactoryContext::http("http://localhost:8080");

// Create a rider with wallet and payment method (3 entities)
let rider = Personas::rider(&mut ctx).await?;
println!("{} <{}>", rider.full_name, rider.email);

// Create a complete booking scenario (7 entities)
let booking = Personas::rider_books_ride(&mut ctx).await?;
println!("Ride {} from {} to {}",
    booking.ride_id,
    booking.pickup_address,
    booking.destination_address);
```

### Available Personas

| Persona | Entities Created | Description |
|---|---|---|
| `rider` | 3 | Verified user + funded wallet + payment method |
| `driver` | 3 | Verified driver user + verified driver profile + wallet |
| `driver_onboard` | 2 | Unverified driver + unverified profile (needs setup) |
| `rider_books_ride` | 7 | Rider (3) + driver (3) + accepted ride |
| `complete_ride` | 7 | Rider + driver + completed ride with payment and ratings |
| `driver_posts_trip` | 4 | Driver (3) + trip post (instant booking) |
| `seed_for_exploration` | 55+ | 3 riders + 4 drivers + 2 bookings + 3 completed rides + 2 trip posts |

### Selective Seeding via API

Use `Personas::seed_via_api` to seed specific scenarios by name:

```rust
let summary = Personas::seed_via_api(
    &mut ctx,
    &["rider", "rider", "driver", "rider-books-ride"],
).await?;

println!("Created {} entities", summary.total_entities_created);
```

Available scenario names: `rider`, `driver`, `driver-onboard`, `rider-books-ride`, `complete-ride`, `driver-posts-trip`, `full`.

---

## Defining Your Own Factory

Here is a complete example defining a factory for an `Article` domain (outside the built-in Ridemate factories):

```rust
use chrono::{DateTime, Utc};
use fabricate::builder::BuildableFactory;
use fabricate::context::FactoryContext;
use fabricate::traits::{FactoryTrait, TraitRegistry};
use fabricate::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 1. Define your entity struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub author_email: String,
    pub status: String,       // "draft", "published", "archived"
    pub view_count: i64,
    pub is_featured: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// 2. Define the factory
pub struct ArticleFactory {
    traits: TraitRegistry<Article>,
}

impl ArticleFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(PublishedTrait));
        traits.register(Box::new(FeaturedTrait));
        traits.register(Box::new(ArchivedTrait));
        Self { traits }
    }
}

// 3. Implement BuildableFactory
impl BuildableFactory<Article> for ArticleFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> Article {
        let n = ctx.sequence("article");
        let now = Utc::now();

        Article {
            id: Uuid::new_v4(),
            title: format!("Test Article {n}"),
            body: format!("This is the body of test article {n}."),
            author_email: ctx.email("author"),
            status: "draft".to_string(),
            view_count: 0,
            is_featured: false,
            published_at: None,
            created_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<Article> {
        &self.traits
    }

    fn apply_overrides(&self, entity: &mut Article, overrides: &[(String, serde_json::Value)]) {
        for (field, value) in overrides {
            match field.as_str() {
                "title" => if let Some(v) = value.as_str() { entity.title = v.to_string(); },
                "body" => if let Some(v) = value.as_str() { entity.body = v.to_string(); },
                "status" => if let Some(v) = value.as_str() { entity.status = v.to_string(); },
                _ => {}
            }
        }
    }

    async fn persist(&self, entity: Article, _ctx: &mut FactoryContext) -> Result<Article> {
        // Add your sqlx INSERT or HTTP POST here
        Ok(entity)
    }
}

// 4. Define traits
struct PublishedTrait;
impl FactoryTrait<Article> for PublishedTrait {
    fn name(&self) -> &str { "published" }
    fn apply(&self, article: &mut Article) {
        article.status = "published".to_string();
        article.published_at = Some(Utc::now());
    }
}

struct FeaturedTrait;
impl FactoryTrait<Article> for FeaturedTrait {
    fn name(&self) -> &str { "featured" }
    fn apply(&self, article: &mut Article) {
        article.status = "published".to_string();
        article.published_at = Some(Utc::now());
        article.is_featured = true;
        article.view_count = 10_000;
    }
}

struct ArchivedTrait;
impl FactoryTrait<Article> for ArchivedTrait {
    fn name(&self) -> &str { "archived" }
    fn apply(&self, article: &mut Article) {
        article.status = "archived".to_string();
    }
}

// 5. Use it
use fabricate::FactoryBuilder;
use serde_json::json;

let mut ctx = /* your FactoryContext */;

let draft = FactoryBuilder::new(ArticleFactory::new())
    .build(&mut ctx)
    .unwrap();
assert_eq!(draft.status, "draft");

let featured = FactoryBuilder::new(ArticleFactory::new())
    .with_trait("featured")
    .set("title", json!("Breaking News"))
    .build(&mut ctx)
    .unwrap();
assert_eq!(featured.status, "published");
assert!(featured.is_featured);
assert_eq!(featured.title, "Breaking News");
```

---

## Built-in Factories Reference

fabricate ships with 11 factories for the Ridemate ride-sharing domain as a reference implementation and for immediate use in Ridemate projects.

| Factory | Output Type | Available Traits |
|---|---|---|
| `UserFactory` | `TestUser` | `verified`, `unverified`, `suspended`, `premium`, `driver`, `admin` |
| `DriverProfileFactory` | `TestDriverProfile` | `verified`, `unverified`, `high_rated`, `new_driver`, `available`, `offline` |
| `RideFactory` | `TestRide` | `requested`, `accepted`, `in_progress`, `completed`, `cancelled`, `with_payment`, `with_ratings` |
| `TripPostFactory` | `TestTripPost` | `instant_book`, `request_to_book`, `recurring`, `with_stops`, `full` |
| `TripBookingFactory` | `TestTripBooking` | `pending`, `confirmed`, `completed`, `cancelled` |
| `PaymentMethodFactory` | `TestPaymentMethod` | `visa`, `mastercard` |
| `WalletFactory` | `TestWallet` | `funded`, `empty` |
| `PaymentFactory` | `TestPayment` | `successful`, `failed`, `refunded`, `pending` |
| `RatingFactory` | `TestRating` | `five_star`, `low_rating`, `with_review`, `driver_rating`, `passenger_rating` |
| `SafetyIncidentFactory` | `TestSafetyIncident` | `panic_button`, `crash_detected`, `resolved` |
| `SafetyContactFactory` | `TestSafetyContact` | *(none)* |

All built-in factories are under `fabricate::ridemate::*` and support all three persistence modes.

---

## CLI Tool

fabricate includes a CLI binary for seeding and managing test data from the command line.

### Installation

```bash
cargo install fabricate
```

Or run directly from the repository:

```bash
cargo run --
```

### Commands

#### `fabricate seed` — Seed test data

```bash
# Seed everything (full exploration dataset)
fabricate seed --target http://localhost:8080

# Seed specific scenarios
fabricate seed --target http://localhost:8080 --scenarios rider,driver,rider-books-ride

# Seed specific personas
fabricate seed --target http://localhost:8080 --personas rider,driver

# Use a custom test API key
fabricate seed --target http://localhost:8080 --test-key my-secret-key
```

#### `fabricate reset` — Delete all test data

```bash
# Reset everything
fabricate reset --target http://localhost:8080

# Reset specific scope
fabricate reset --target http://localhost:8080 --scope users
```

#### `fabricate list` — Show available factories and personas

```bash
fabricate list
```

Output:

```
fabricate: Available Factories & Personas

FACTORIES:
  UserFactory
    Traits: verified, unverified, suspended, premium, driver, admin
  DriverProfileFactory
    Traits: verified, unverified, high_rated, new_driver, available, offline
  RideFactory
    Traits: requested, accepted, in_progress, completed, cancelled, with_payment, with_ratings
  ...

PERSONAS (scenario bundles):
  rider                  - User + wallet + payment method
  driver                 - User (driver) + driver profile + wallet
  driver-onboard         - Driver (unverified, needs setup)
  rider-books-ride       - Rider + driver + accepted ride
  complete-ride          - Rider + driver + completed ride + payment + ratings
  driver-posts-trip      - Driver + trip post (carpooling)
  full                   - All of the above (comprehensive exploration data)
```

---

## Architecture Overview

```
                    ┌─────────────────────┐
                    │    FactoryBuilder    │  Fluent API: .with_trait() .set() .build() .create()
                    └──────────┬──────────┘
                               │ uses
                    ┌──────────▼──────────┐
                    │  BuildableFactory<T> │  build_base() + trait_registry() + apply_overrides() + persist()
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
    ┌─────────▼───────┐ ┌─────▼──────┐ ┌───────▼────────┐
    │  TraitRegistry   │ │  Sequence   │ │  FactoryContext │
    │  FactoryTrait<T> │ │  (counters) │ │  (pool/http/   │
    │  (named mods)    │ │             │ │   overrides)   │
    └─────────────────┘ └────────────┘ └───────┬────────┘
                                                │ persistence via
                                   ┌────────────┼────────────┐
                                   │            │            │
                              ┌────▼───┐  ┌─────▼────┐  ┌───▼──┐
                              │ sqlx   │  │ reqwest  │  │ None │
                              │ PgPool │  │ HTTP API │  │ (mem)│
                              └────────┘  └──────────┘  └──────┘
```

| Type | Role |
|---|---|
| `FactoryBuilder<F, T>` | Fluent builder that chains traits, overrides, and persistence |
| `BuildableFactory<T>` | Trait defining how to build, customize, and persist an entity |
| `Factory` | Higher-level trait with `build`, `create`, `create_list` |
| `FactoryContext` | Session context holding DB pool, HTTP client, sequences, and overrides |
| `Sequence` | Named auto-incrementing counters with helpers for emails, phones, names, plates |
| `TraitRegistry<T>` | Registry of named `FactoryTrait<T>` implementations for an entity |
| `FactoryTrait<T>` | A named modification (e.g., "verified") that mutates an entity |
| `Association` | Describes a foreign key dependency between factories |
| `Personas` | Pre-composed multi-entity scenario bundles |

---

## Cargo Features

| Feature | Default | Description |
|---|---|---|
| `postgres` | Yes | Enables direct Postgres persistence via sqlx (`FactoryContext::database()`) |

To disable the default `postgres` feature (HTTP-only or build-only usage):

```toml
[dev-dependencies]
fabricate = { version = "0.1", default-features = false }
```

---

## Error Handling

All fallible operations return `fabricate::Result<T>`, which is `std::result::Result<T, fabricate::Error>`.

| Variant | Description |
|---|---|
| `Error::Build(String)` | Factory construction failed (missing config, invalid state) |
| `Error::Association(String)` | Associated entity could not be resolved |
| `Error::TraitNotFound(String)` | Requested trait is not registered on the factory |
| `Error::Sequence(String)` | Sequence generation error |
| `Error::Database(sqlx::Error)` | Database operation failed (postgres feature only) |
| `Error::Http(reqwest::Error)` | HTTP request to test API failed |
| `Error::Json(serde_json::Error)` | JSON serialization/deserialization error |
| `Error::Persona(String)` | Unknown persona or scenario name |

Errors include helpful context. For example, requesting a non-existent trait:

```
Trait 'nonexistent_trait' not registered. Available: ["verified", "unverified", "suspended", "premium", "driver", "admin"]
```

---

## TypeScript Client

A TypeScript companion client is available in the `typescript/` directory. It provides the same entity creation capabilities via HTTP API calls to your backend's `/__test__/*` endpoints.

### Quick Start

```bash
cd typescript
npm install
```

### Usage

```typescript
import { Factory } from './src/index.js';

const factory = new Factory();

// Create entities
const rider = await factory.create('rider');
const driver = await factory.create('driver');
const ride = await factory.create('ride');

// Access auth tokens
const token = factory.getAuthToken('rider');

// Reset database between scenarios
await factory.reset();
```

### Environment Variables

- `SENTINEL_BACKEND_URL` — Backend test API base URL (default: `http://localhost:8080/api/v1/test`)
- `TEST_API_KEY` — API key for test endpoints (default: `test-key`)

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b my-feature`)
3. Make your changes
4. Run tests and checks:
   ```bash
   cargo test
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```
5. Commit with a descriptive message
6. Open a Pull Request

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

fabricate is inspired by [FactoryBot](https://github.com/thoughtbot/factory_bot) by [thoughtbot](https://thoughtbot.com/), the gold standard for test data factories in the Ruby ecosystem. This project aims to bring the same developer experience to Rust.
