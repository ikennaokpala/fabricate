use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::builder::BuildableFactory;
use crate::context::FactoryContext;
use crate::traits::{FactoryTrait, TraitRegistry};
use crate::Result;

/// Safety incident entity matching the backend `safety_incidents` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSafetyIncident {
    pub id: Uuid,
    pub ride_id: Uuid,
    pub user_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub incident_type: String,
    pub status: String,
    pub severity: i32,
    pub title: String,
    pub description: String,
    pub incident_address: Option<String>,
    pub emergency_contact_notified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Safety contact entity matching the backend `safety_contacts` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSafetyContact {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub relationship: String,
    pub phone_number: String,
    pub email: Option<String>,
    pub is_primary: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- SafetyIncidentFactory ---

pub struct SafetyIncidentFactory {
    traits: TraitRegistry<TestSafetyIncident>,
}

impl SafetyIncidentFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(PanicButtonTrait));
        traits.register(Box::new(CrashDetectedTrait));
        traits.register(Box::new(ResolvedIncidentTrait));
        Self { traits }
    }
}

impl Default for SafetyIncidentFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestSafetyIncident> for SafetyIncidentFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestSafetyIncident {
        let _n = ctx.sequence("incident");
        let now = Utc::now();

        TestSafetyIncident {
            id: Uuid::new_v4(),
            ride_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            driver_id: None,
            incident_type: "panic_button".to_string(),
            status: "active".to_string(),
            severity: 10,
            title: "Emergency SOS activated".to_string(),
            description: "Passenger activated panic button during ride".to_string(),
            incident_address: Some("123 Market St, San Francisco, CA".to_string()),
            emergency_contact_notified: false,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestSafetyIncident> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestSafetyIncident,
        overrides: &[(String, serde_json::Value)],
    ) {
        for (field, value) in overrides {
            match field.as_str() {
                "ride_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.ride_id = uuid;
                        }
                    }
                }
                "user_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.user_id = uuid;
                        }
                    }
                }
                "incident_type" => {
                    if let Some(v) = value.as_str() {
                        entity.incident_type = v.to_string();
                    }
                }
                "severity" => {
                    if let Some(v) = value.as_i64() {
                        entity.severity = v as i32;
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(
        &self,
        entity: TestSafetyIncident,
        ctx: &mut FactoryContext,
    ) -> Result<TestSafetyIncident> {
        // HTTP API mode
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "ride_id": entity.ride_id.to_string(),
                "user_id": entity.user_id.to_string(),
                "driver_id": entity.driver_id.map(|id| id.to_string()),
                "incident_type": entity.incident_type,
                "status": entity.status,
                "severity": entity.severity,
                "title": entity.title,
                "description": entity.description,
                "incident_address": entity.incident_address,
                "emergency_contact_notified": entity.emergency_contact_notified,
            });
            let resp = ctx.test_post("/__test__/safety-incidents", &body).await?;
            let mut result = entity.clone();
            if let Some(id) = resp.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = Uuid::parse_str(id) {
                    result.id = uuid;
                }
            }
            return Ok(result);
        }

        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            sqlx::query(
                r#"
                INSERT INTO safety_incidents (
                    id, ride_id, user_id, driver_id, incident_type, status, severity,
                    title, description, incident_address, emergency_contact_notified,
                    response_actions, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5::incident_type, $6::incident_status, $7, $8, $9, $10, $11, '[]'::jsonb, $12, $13)
                "#,
            )
            .bind(entity.id)
            .bind(entity.ride_id)
            .bind(entity.user_id)
            .bind(entity.driver_id)
            .bind(&entity.incident_type)
            .bind(&entity.status)
            .bind(entity.severity)
            .bind(&entity.title)
            .bind(&entity.description)
            .bind(&entity.incident_address)
            .bind(entity.emergency_contact_notified)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- SafetyContactFactory ---

pub struct SafetyContactFactory {
    traits: TraitRegistry<TestSafetyContact>,
}

impl SafetyContactFactory {
    pub fn new() -> Self {
        let traits = TraitRegistry::new();
        Self { traits }
    }
}

impl Default for SafetyContactFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestSafetyContact> for SafetyContactFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestSafetyContact {
        let n = ctx.sequence("safety_contact");
        let now = Utc::now();

        TestSafetyContact {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: format!("Emergency Contact {n}"),
            relationship: "family".to_string(),
            phone_number: ctx.phone(),
            email: Some(ctx.email("emergency")),
            is_primary: n == 1,
            is_verified: true,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestSafetyContact> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestSafetyContact,
        overrides: &[(String, serde_json::Value)],
    ) {
        for (field, value) in overrides {
            match field.as_str() {
                "user_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.user_id = uuid;
                        }
                    }
                }
                "name" => {
                    if let Some(v) = value.as_str() {
                        entity.name = v.to_string();
                    }
                }
                "relationship" => {
                    if let Some(v) = value.as_str() {
                        entity.relationship = v.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(
        &self,
        entity: TestSafetyContact,
        ctx: &mut FactoryContext,
    ) -> Result<TestSafetyContact> {
        // HTTP API mode
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "user_id": entity.user_id.to_string(),
                "name": entity.name,
                "relationship": entity.relationship,
                "phone_number": entity.phone_number,
                "email": entity.email,
                "is_primary": entity.is_primary,
                "is_verified": entity.is_verified,
            });
            let resp = ctx.test_post("/__test__/safety-contacts", &body).await?;
            let mut result = entity.clone();
            if let Some(id) = resp.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = Uuid::parse_str(id) {
                    result.id = uuid;
                }
            }
            return Ok(result);
        }

        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            sqlx::query(
                r#"
                INSERT INTO safety_contacts (
                    id, user_id, name, relationship, phone_number, email,
                    is_primary, is_verified, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(entity.id)
            .bind(entity.user_id)
            .bind(&entity.name)
            .bind(&entity.relationship)
            .bind(&entity.phone_number)
            .bind(&entity.email)
            .bind(entity.is_primary)
            .bind(entity.is_verified)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- Traits ---

struct PanicButtonTrait;
impl FactoryTrait<TestSafetyIncident> for PanicButtonTrait {
    fn name(&self) -> &str {
        "panic_button"
    }
    fn apply(&self, incident: &mut TestSafetyIncident) {
        incident.incident_type = "panic_button".to_string();
        incident.severity = 10;
        incident.status = "active".to_string();
    }
}

struct CrashDetectedTrait;
impl FactoryTrait<TestSafetyIncident> for CrashDetectedTrait {
    fn name(&self) -> &str {
        "crash_detected"
    }
    fn apply(&self, incident: &mut TestSafetyIncident) {
        incident.incident_type = "crash_detected".to_string();
        incident.severity = 10;
        incident.title = "Crash detected by accelerometer".to_string();
        incident.description = "Vehicle impact detected via device sensors".to_string();
    }
}

struct ResolvedIncidentTrait;
impl FactoryTrait<TestSafetyIncident> for ResolvedIncidentTrait {
    fn name(&self) -> &str {
        "resolved"
    }
    fn apply(&self, incident: &mut TestSafetyIncident) {
        incident.status = "resolved".to_string();
    }
}
