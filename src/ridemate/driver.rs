use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::builder::BuildableFactory;
use crate::context::FactoryContext;
use crate::traits::{FactoryTrait, TraitRegistry};
use crate::Result;

use super::COLORS;
use super::VEHICLES;

/// Driver profile entity matching the backend `driver_profiles` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDriverProfile {
    pub user_id: Uuid,
    pub license_number: String,
    pub license_expiry_date: NaiveDate,
    pub is_license_verified: bool,
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub vehicle_color: String,
    pub vehicle_plate_number: String,
    pub is_insurance_verified: bool,
    pub insurance_policy_number: String,
    pub insurance_provider: String,
    pub insurance_expiry_date: NaiveDate,
    pub average_rating: f64,
    pub total_trips: i32,
    pub total_ratings: i32,
    pub acceptance_rate: f64,
    pub cancellation_rate: f64,
    pub is_available: bool,
    pub is_online: bool,
    pub background_check_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Factory for creating test driver profiles.
pub struct DriverProfileFactory {
    traits: TraitRegistry<TestDriverProfile>,
}

impl DriverProfileFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(VerifiedDriverTrait));
        traits.register(Box::new(UnverifiedDriverTrait));
        traits.register(Box::new(HighRatedDriverTrait));
        traits.register(Box::new(NewDriverTrait));
        traits.register(Box::new(AvailableDriverTrait));
        traits.register(Box::new(OfflineDriverTrait));
        Self { traits }
    }
}

impl Default for DriverProfileFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestDriverProfile> for DriverProfileFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestDriverProfile {
        let n = ctx.sequence("driver");
        let vehicle_idx = (n as usize - 1) % VEHICLES.len();
        let (make, model) = VEHICLES[vehicle_idx];
        let color = COLORS[(n as usize - 1) % COLORS.len()];
        let plate = ctx.sequences.plate();
        let now = Utc::now();
        let expiry = NaiveDate::from_ymd_opt(2027, 12, 31).unwrap();

        TestDriverProfile {
            user_id: Uuid::new_v4(),
            license_number: format!("DL{n:06}"),
            license_expiry_date: expiry,
            is_license_verified: true,
            vehicle_make: make.to_string(),
            vehicle_model: model.to_string(),
            vehicle_year: 2020 + (n as i32 % 5),
            vehicle_color: color.to_string(),
            vehicle_plate_number: plate,
            is_insurance_verified: true,
            insurance_policy_number: format!("INS{n:06}"),
            insurance_provider: "StateFarm".to_string(),
            insurance_expiry_date: expiry,
            average_rating: 4.5,
            total_trips: 50,
            total_ratings: 40,
            acceptance_rate: 92.0,
            cancellation_rate: 3.0,
            is_available: true,
            is_online: true,
            background_check_status: "completed".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestDriverProfile> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestDriverProfile,
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
                "average_rating" => {
                    if let Some(v) = value.as_f64() {
                        entity.average_rating = v;
                    }
                }
                "total_trips" => {
                    if let Some(v) = value.as_i64() {
                        entity.total_trips = v as i32;
                    }
                }
                "vehicle_make" => {
                    if let Some(v) = value.as_str() {
                        entity.vehicle_make = v.to_string();
                    }
                }
                "vehicle_model" => {
                    if let Some(v) = value.as_str() {
                        entity.vehicle_model = v.to_string();
                    }
                }
                "is_available" => {
                    if let Some(v) = value.as_bool() {
                        entity.is_available = v;
                    }
                }
                "is_online" => {
                    if let Some(v) = value.as_bool() {
                        entity.is_online = v;
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(
        &self,
        entity: TestDriverProfile,
        ctx: &mut FactoryContext,
    ) -> Result<TestDriverProfile> {
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "user_id": entity.user_id.to_string(),
                "vehicle_make": entity.vehicle_make,
                "vehicle_model": entity.vehicle_model,
                "vehicle_year": entity.vehicle_year,
                "vehicle_color": entity.vehicle_color,
                "vehicle_plate_number": entity.vehicle_plate_number,
                "license_number": entity.license_number,
                "is_verified": entity.is_license_verified,
                "average_rating": entity.average_rating,
                "total_trips": entity.total_trips,
            });
            ctx.test_post("/__test__/drivers", &body).await?;
            return Ok(entity);
        }

        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            sqlx::query(
                r#"
                INSERT INTO driver_profiles (
                    user_id, license_number, license_expiry_date, is_license_verified,
                    vehicle_make, vehicle_model, vehicle_year, vehicle_color, vehicle_plate_number,
                    is_insurance_verified, insurance_policy_number, insurance_provider, insurance_expiry_date,
                    average_rating, total_trips, total_ratings, acceptance_rate, cancellation_rate,
                    is_available, is_online, background_check_status, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
                ON CONFLICT (user_id) DO UPDATE SET updated_at = $23
                "#,
            )
            .bind(entity.user_id)
            .bind(&entity.license_number)
            .bind(entity.license_expiry_date)
            .bind(entity.is_license_verified)
            .bind(&entity.vehicle_make)
            .bind(&entity.vehicle_model)
            .bind(entity.vehicle_year)
            .bind(&entity.vehicle_color)
            .bind(&entity.vehicle_plate_number)
            .bind(entity.is_insurance_verified)
            .bind(&entity.insurance_policy_number)
            .bind(&entity.insurance_provider)
            .bind(entity.insurance_expiry_date)
            .bind(entity.average_rating)
            .bind(entity.total_trips)
            .bind(entity.total_ratings)
            .bind(entity.acceptance_rate)
            .bind(entity.cancellation_rate)
            .bind(entity.is_available)
            .bind(entity.is_online)
            .bind(&entity.background_check_status)
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

struct VerifiedDriverTrait;
impl FactoryTrait<TestDriverProfile> for VerifiedDriverTrait {
    fn name(&self) -> &str {
        "verified"
    }
    fn apply(&self, dp: &mut TestDriverProfile) {
        dp.is_license_verified = true;
        dp.is_insurance_verified = true;
        dp.background_check_status = "completed".to_string();
    }
}

struct UnverifiedDriverTrait;
impl FactoryTrait<TestDriverProfile> for UnverifiedDriverTrait {
    fn name(&self) -> &str {
        "unverified"
    }
    fn apply(&self, dp: &mut TestDriverProfile) {
        dp.is_license_verified = false;
        dp.is_insurance_verified = false;
        dp.background_check_status = "pending".to_string();
        dp.is_available = false;
        dp.is_online = false;
    }
}

struct HighRatedDriverTrait;
impl FactoryTrait<TestDriverProfile> for HighRatedDriverTrait {
    fn name(&self) -> &str {
        "high_rated"
    }
    fn apply(&self, dp: &mut TestDriverProfile) {
        dp.average_rating = 4.9;
        dp.total_trips = 250;
        dp.total_ratings = 200;
        dp.acceptance_rate = 98.0;
        dp.cancellation_rate = 1.0;
    }
}

struct NewDriverTrait;
impl FactoryTrait<TestDriverProfile> for NewDriverTrait {
    fn name(&self) -> &str {
        "new_driver"
    }
    fn apply(&self, dp: &mut TestDriverProfile) {
        dp.average_rating = 0.0;
        dp.total_trips = 0;
        dp.total_ratings = 0;
        dp.acceptance_rate = 100.0;
        dp.cancellation_rate = 0.0;
    }
}

struct AvailableDriverTrait;
impl FactoryTrait<TestDriverProfile> for AvailableDriverTrait {
    fn name(&self) -> &str {
        "available"
    }
    fn apply(&self, dp: &mut TestDriverProfile) {
        dp.is_available = true;
        dp.is_online = true;
    }
}

struct OfflineDriverTrait;
impl FactoryTrait<TestDriverProfile> for OfflineDriverTrait {
    fn name(&self) -> &str {
        "offline"
    }
    fn apply(&self, dp: &mut TestDriverProfile) {
        dp.is_available = false;
        dp.is_online = false;
    }
}
