use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::builder::BuildableFactory;
use crate::context::FactoryContext;
use crate::traits::{FactoryTrait, TraitRegistry};
use crate::Result;

/// User entity matching the backend `users` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub phone_number: String,
    pub password_hash: String,
    pub full_name: String,
    pub user_type: String,
    pub role: String,
    pub is_email_verified: bool,
    pub is_phone_verified: bool,
    pub is_active: bool,
    pub bio: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Factory for creating test users.
pub struct UserFactory {
    traits: TraitRegistry<TestUser>,
}

impl UserFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(VerifiedTrait));
        traits.register(Box::new(UnverifiedTrait));
        traits.register(Box::new(SuspendedTrait));
        traits.register(Box::new(PremiumTrait));
        traits.register(Box::new(DriverTypeTrait));
        traits.register(Box::new(AdminTrait));
        Self { traits }
    }
}

impl Default for UserFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestUser> for UserFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestUser {
        let email = ctx.email("user");
        let phone = ctx.phone();
        let name = ctx.full_name();
        let now = Utc::now();

        TestUser {
            id: Uuid::new_v4(),
            email,
            phone_number: phone,
            // bcrypt hash of "testpassword123"
            password_hash: "$2b$12$LJ3m4ys3Lg0UbSDqr5dU9ODkHVEwVyFHFPZqJOSJmgFwYfqBqnmFy"
                .to_string(),
            full_name: name,
            user_type: "passenger".to_string(),
            role: "user".to_string(),
            is_email_verified: true,
            is_phone_verified: true,
            is_active: true,
            bio: None,
            photo_url: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestUser> {
        &self.traits
    }

    fn apply_overrides(&self, entity: &mut TestUser, overrides: &[(String, serde_json::Value)]) {
        for (field, value) in overrides {
            match field.as_str() {
                "email" => {
                    if let Some(v) = value.as_str() {
                        entity.email = v.to_string();
                    }
                }
                "full_name" => {
                    if let Some(v) = value.as_str() {
                        entity.full_name = v.to_string();
                    }
                }
                "user_type" => {
                    if let Some(v) = value.as_str() {
                        entity.user_type = v.to_string();
                    }
                }
                "phone_number" => {
                    if let Some(v) = value.as_str() {
                        entity.phone_number = v.to_string();
                    }
                }
                "id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.id = uuid;
                        }
                    }
                }
                "is_active" => {
                    if let Some(v) = value.as_bool() {
                        entity.is_active = v;
                    }
                }
                "role" => {
                    if let Some(v) = value.as_str() {
                        entity.role = v.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(&self, entity: TestUser, ctx: &mut FactoryContext) -> Result<TestUser> {
        // HTTP API mode
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "email": entity.email,
                "phone_number": entity.phone_number,
                "full_name": entity.full_name,
                "user_type": entity.user_type,
                "password": "testpassword123",
            });
            let resp = ctx.test_post("/__test__/users", &body).await?;

            // Extract the created user ID from response if available
            let mut result = entity.clone();
            if let Some(id) = resp.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = Uuid::parse_str(id) {
                    result.id = uuid;
                }
            }
            return Ok(result);
        }

        // Direct DB mode
        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            let row = sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO users (id, email, phone_number, password_hash, full_name, user_type, role, is_email_verified, is_phone_verified, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (email) DO UPDATE SET updated_at = $12
                RETURNING id
                "#,
            )
            .bind(entity.id)
            .bind(&entity.email)
            .bind(&entity.phone_number)
            .bind(&entity.password_hash)
            .bind(&entity.full_name)
            .bind(&entity.user_type)
            .bind(&entity.role)
            .bind(entity.is_email_verified)
            .bind(entity.is_phone_verified)
            .bind(entity.is_active)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .fetch_one(pool)
            .await?;

            let mut result = entity;
            result.id = row;
            return Ok(result);
        }

        Ok(entity)
    }
}

// --- Traits ---

struct VerifiedTrait;
impl FactoryTrait<TestUser> for VerifiedTrait {
    fn name(&self) -> &str {
        "verified"
    }
    fn apply(&self, user: &mut TestUser) {
        user.is_email_verified = true;
        user.is_phone_verified = true;
    }
}

struct UnverifiedTrait;
impl FactoryTrait<TestUser> for UnverifiedTrait {
    fn name(&self) -> &str {
        "unverified"
    }
    fn apply(&self, user: &mut TestUser) {
        user.is_email_verified = false;
        user.is_phone_verified = false;
    }
}

struct SuspendedTrait;
impl FactoryTrait<TestUser> for SuspendedTrait {
    fn name(&self) -> &str {
        "suspended"
    }
    fn apply(&self, user: &mut TestUser) {
        user.is_active = false;
    }
}

struct PremiumTrait;
impl FactoryTrait<TestUser> for PremiumTrait {
    fn name(&self) -> &str {
        "premium"
    }
    fn apply(&self, user: &mut TestUser) {
        user.is_email_verified = true;
        user.is_phone_verified = true;
        user.bio = Some("Premium member since 2024".to_string());
    }
}

struct DriverTypeTrait;
impl FactoryTrait<TestUser> for DriverTypeTrait {
    fn name(&self) -> &str {
        "driver"
    }
    fn apply(&self, user: &mut TestUser) {
        user.user_type = "driver".to_string();
    }
}

struct AdminTrait;
impl FactoryTrait<TestUser> for AdminTrait {
    fn name(&self) -> &str {
        "admin"
    }
    fn apply(&self, user: &mut TestUser) {
        user.user_type = "passenger".to_string();
        user.role = "admin".to_string();
    }
}
