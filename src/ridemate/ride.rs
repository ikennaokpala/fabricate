use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::builder::BuildableFactory;
use crate::context::FactoryContext;
use crate::traits::{FactoryTrait, TraitRegistry};
use crate::Result;

use super::{test_address, test_latitude, test_longitude};

/// Ride entity matching the backend `rides` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRide {
    pub id: Uuid,
    pub passenger_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub ride_type: String,
    pub status: String,
    pub pickup_latitude: f64,
    pub pickup_longitude: f64,
    pub pickup_address: String,
    pub pickup_place_name: String,
    pub destination_latitude: f64,
    pub destination_longitude: f64,
    pub destination_address: String,
    pub destination_place_name: String,
    pub estimated_price_cents: i64,
    pub final_price_cents: Option<i64>,
    pub base_fare_cents: i64,
    pub distance_fare_cents: i64,
    pub time_fare_cents: i64,
    pub surge_multiplier: f64,
    pub distance_km: f64,
    pub estimated_duration_minutes: i32,
    pub actual_duration_minutes: Option<i32>,
    pub requested_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub driver_en_route_at: Option<DateTime<Utc>>,
    pub driver_arrived_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub payment_status: String,
    pub payment_intent_id: Option<String>,
    pub payment_method_id: Option<String>,
    pub cancellation_reason: Option<String>,
    pub cancelled_by: Option<String>,
    pub passenger_rating: Option<i32>,
    pub driver_rating: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Factory for creating test rides.
pub struct RideFactory {
    traits: TraitRegistry<TestRide>,
}

impl RideFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(RequestedTrait));
        traits.register(Box::new(AcceptedTrait));
        traits.register(Box::new(InProgressTrait));
        traits.register(Box::new(CompletedTrait));
        traits.register(Box::new(CancelledTrait));
        traits.register(Box::new(WithPaymentTrait));
        traits.register(Box::new(WithRatingsTrait));
        Self { traits }
    }
}

impl Default for RideFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestRide> for RideFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestRide {
        let n = ctx.sequence("ride");
        let now = Utc::now();

        TestRide {
            id: Uuid::new_v4(),
            passenger_id: Uuid::new_v4(),
            driver_id: None,
            ride_type: "economy".to_string(),
            status: "requested".to_string(),
            pickup_latitude: test_latitude(n),
            pickup_longitude: test_longitude(n),
            pickup_address: test_address(n),
            pickup_place_name: "Pickup".to_string(),
            destination_latitude: test_latitude(n + 100),
            destination_longitude: test_longitude(n + 100),
            destination_address: test_address(n + 100),
            destination_place_name: "Destination".to_string(),
            estimated_price_cents: 2500,
            final_price_cents: None,
            base_fare_cents: 500,
            distance_fare_cents: 1500,
            time_fare_cents: 500,
            surge_multiplier: 1.0,
            distance_km: 10.0,
            estimated_duration_minutes: 15,
            actual_duration_minutes: None,
            requested_at: now,
            accepted_at: None,
            driver_en_route_at: None,
            driver_arrived_at: None,
            started_at: None,
            completed_at: None,
            cancelled_at: None,
            payment_status: "pending".to_string(),
            payment_intent_id: None,
            payment_method_id: None,
            cancellation_reason: None,
            cancelled_by: None,
            passenger_rating: None,
            driver_rating: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestRide> {
        &self.traits
    }

    fn apply_overrides(&self, entity: &mut TestRide, overrides: &[(String, serde_json::Value)]) {
        for (field, value) in overrides {
            match field.as_str() {
                "passenger_id" | "rider_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.passenger_id = uuid;
                        }
                    }
                }
                "driver_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.driver_id = Some(uuid);
                        }
                    }
                }
                "status" => {
                    if let Some(v) = value.as_str() {
                        entity.status = v.to_string();
                    }
                }
                "ride_type" => {
                    if let Some(v) = value.as_str() {
                        entity.ride_type = v.to_string();
                    }
                }
                "estimated_price_cents" => {
                    if let Some(v) = value.as_i64() {
                        entity.estimated_price_cents = v;
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(&self, entity: TestRide, ctx: &mut FactoryContext) -> Result<TestRide> {
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "passenger_id": entity.passenger_id.to_string(),
                "driver_id": entity.driver_id.map(|id| id.to_string()),
                "ride_type": entity.ride_type,
                "status": entity.status,
                "pickup_latitude": entity.pickup_latitude,
                "pickup_longitude": entity.pickup_longitude,
                "pickup_address": entity.pickup_address,
                "destination_latitude": entity.destination_latitude,
                "destination_longitude": entity.destination_longitude,
                "destination_address": entity.destination_address,
                "estimated_price_cents": entity.estimated_price_cents,
                "payment_status": entity.payment_status,
            });
            let resp = ctx.test_post("/__test__/rides", &body).await?;
            let mut result = entity;
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
                INSERT INTO rides (
                    id, passenger_id, driver_id, ride_type, status,
                    pickup_location, pickup_address, pickup_place_name,
                    destination_location, destination_address, destination_place_name,
                    estimated_price_cents, final_price_cents, base_fare_cents, distance_fare_cents, time_fare_cents,
                    surge_multiplier, distance_km, estimated_duration_minutes, actual_duration_minutes,
                    requested_at, accepted_at, started_at, completed_at, cancelled_at,
                    payment_status, payment_intent_id, payment_method_id,
                    passenger_rating, driver_rating,
                    created_at, updated_at
                )
                VALUES (
                    $1, $2, $3, $4::ride_type, $5::ride_status,
                    ST_SetSRID(ST_MakePoint($6, $7), 4326)::geography, $8, $9,
                    ST_SetSRID(ST_MakePoint($10, $11), 4326)::geography, $12, $13,
                    $14, $15, $16, $17, $18,
                    $19, $20, $21, $22,
                    $23, $24, $25, $26, $27,
                    $28, $29, $30,
                    $31, $32,
                    $33, $34
                )
                "#,
            )
            .bind(entity.id)
            .bind(entity.passenger_id)
            .bind(entity.driver_id)
            .bind(&entity.ride_type)
            .bind(&entity.status)
            .bind(entity.pickup_longitude)
            .bind(entity.pickup_latitude)
            .bind(&entity.pickup_address)
            .bind(&entity.pickup_place_name)
            .bind(entity.destination_longitude)
            .bind(entity.destination_latitude)
            .bind(&entity.destination_address)
            .bind(&entity.destination_place_name)
            .bind(entity.estimated_price_cents)
            .bind(entity.final_price_cents)
            .bind(entity.base_fare_cents)
            .bind(entity.distance_fare_cents)
            .bind(entity.time_fare_cents)
            .bind(entity.surge_multiplier)
            .bind(entity.distance_km)
            .bind(entity.estimated_duration_minutes)
            .bind(entity.actual_duration_minutes)
            .bind(entity.requested_at)
            .bind(entity.accepted_at)
            .bind(entity.started_at)
            .bind(entity.completed_at)
            .bind(entity.cancelled_at)
            .bind(&entity.payment_status)
            .bind(&entity.payment_intent_id)
            .bind(&entity.payment_method_id)
            .bind(entity.passenger_rating)
            .bind(entity.driver_rating)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- Traits (ride status FSM) ---

struct RequestedTrait;
impl FactoryTrait<TestRide> for RequestedTrait {
    fn name(&self) -> &str {
        "requested"
    }
    fn apply(&self, ride: &mut TestRide) {
        ride.status = "requested".to_string();
        ride.driver_id = None;
        ride.accepted_at = None;
        ride.started_at = None;
        ride.completed_at = None;
    }
}

struct AcceptedTrait;
impl FactoryTrait<TestRide> for AcceptedTrait {
    fn name(&self) -> &str {
        "accepted"
    }
    fn apply(&self, ride: &mut TestRide) {
        ride.status = "accepted".to_string();
        let now = Utc::now();
        ride.accepted_at = Some(now - Duration::minutes(5));
    }
}

struct InProgressTrait;
impl FactoryTrait<TestRide> for InProgressTrait {
    fn name(&self) -> &str {
        "in_progress"
    }
    fn apply(&self, ride: &mut TestRide) {
        ride.status = "in_progress".to_string();
        let now = Utc::now();
        ride.accepted_at = Some(now - Duration::minutes(20));
        ride.driver_en_route_at = Some(now - Duration::minutes(15));
        ride.driver_arrived_at = Some(now - Duration::minutes(10));
        ride.started_at = Some(now - Duration::minutes(5));
    }
}

struct CompletedTrait;
impl FactoryTrait<TestRide> for CompletedTrait {
    fn name(&self) -> &str {
        "completed"
    }
    fn apply(&self, ride: &mut TestRide) {
        ride.status = "completed".to_string();
        let now = Utc::now();
        ride.accepted_at = Some(now - Duration::minutes(30));
        ride.driver_en_route_at = Some(now - Duration::minutes(25));
        ride.driver_arrived_at = Some(now - Duration::minutes(20));
        ride.started_at = Some(now - Duration::minutes(15));
        ride.completed_at = Some(now);
        ride.actual_duration_minutes = Some(15);
        ride.final_price_cents = Some(ride.estimated_price_cents);
        ride.payment_status = "succeeded".to_string();
        ride.payment_intent_id = Some(format!("pi_test_{}", Uuid::new_v4()));
    }
}

struct CancelledTrait;
impl FactoryTrait<TestRide> for CancelledTrait {
    fn name(&self) -> &str {
        "cancelled"
    }
    fn apply(&self, ride: &mut TestRide) {
        ride.status = "cancelled".to_string();
        ride.cancelled_at = Some(Utc::now());
        ride.cancelled_by = Some("passenger".to_string());
        ride.cancellation_reason = Some("Changed plans".to_string());
    }
}

struct WithPaymentTrait;
impl FactoryTrait<TestRide> for WithPaymentTrait {
    fn name(&self) -> &str {
        "with_payment"
    }
    fn apply(&self, ride: &mut TestRide) {
        ride.payment_status = "succeeded".to_string();
        ride.payment_intent_id = Some(format!("pi_test_{}", Uuid::new_v4()));
        ride.payment_method_id = Some("pm_card_visa".to_string());
        ride.final_price_cents = Some(ride.estimated_price_cents);
    }
}

struct WithRatingsTrait;
impl FactoryTrait<TestRide> for WithRatingsTrait {
    fn name(&self) -> &str {
        "with_ratings"
    }
    fn apply(&self, ride: &mut TestRide) {
        ride.passenger_rating = Some(5);
        ride.driver_rating = Some(5);
    }
}
