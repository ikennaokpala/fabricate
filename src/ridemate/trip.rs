use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::builder::BuildableFactory;
use crate::context::FactoryContext;
use crate::traits::{FactoryTrait, TraitRegistry};
use crate::Result;

use super::{test_address, test_latitude, test_longitude};

/// Trip post entity matching the backend `trip_posts` table (carpooling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTripPost {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub origin_address: String,
    pub origin_lat: f64,
    pub origin_lng: f64,
    pub destination_address: String,
    pub destination_lat: f64,
    pub destination_lng: f64,
    pub departure_time: DateTime<Utc>,
    pub total_seats: i32,
    pub available_seats: i32,
    pub price_per_seat_cents: i64,
    pub booking_mode: String,
    pub distance_km: Option<f64>,
    pub duration_minutes: Option<i32>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Trip booking entity matching the backend `trip_bookings` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTripBooking {
    pub id: Uuid,
    pub trip_id: Uuid,
    pub passenger_id: Uuid,
    pub seats_booked: i32,
    pub total_price_cents: i64,
    pub status: String,
    pub pickup_point_address: Option<String>,
    pub pickup_point_lat: Option<f64>,
    pub pickup_point_lng: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- TripPostFactory ---

pub struct TripPostFactory {
    traits: TraitRegistry<TestTripPost>,
}

impl TripPostFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(InstantBookTrait));
        traits.register(Box::new(RequestToBookTrait));
        traits.register(Box::new(RecurringTripTrait));
        traits.register(Box::new(WithStopsTrait));
        traits.register(Box::new(FullTripTrait));
        Self { traits }
    }
}

impl Default for TripPostFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestTripPost> for TripPostFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestTripPost {
        let n = ctx.sequence("trip");
        let now = Utc::now();
        let departure = now + Duration::hours(24);

        TestTripPost {
            id: Uuid::new_v4(),
            driver_id: Uuid::new_v4(),
            origin_address: test_address(n),
            origin_lat: test_latitude(n),
            origin_lng: test_longitude(n),
            destination_address: test_address(n + 50),
            destination_lat: test_latitude(n + 50),
            destination_lng: test_longitude(n + 50),
            departure_time: departure,
            total_seats: 3,
            available_seats: 3,
            price_per_seat_cents: 1500,
            booking_mode: "instant".to_string(),
            distance_km: Some(45.0),
            duration_minutes: Some(50),
            status: "upcoming".to_string(),
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestTripPost> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestTripPost,
        overrides: &[(String, serde_json::Value)],
    ) {
        for (field, value) in overrides {
            match field.as_str() {
                "driver_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.driver_id = uuid;
                        }
                    }
                }
                "price_per_seat_cents" => {
                    if let Some(v) = value.as_i64() {
                        entity.price_per_seat_cents = v;
                    }
                }
                "total_seats" => {
                    if let Some(v) = value.as_i64() {
                        entity.total_seats = v as i32;
                        entity.available_seats = v as i32;
                    }
                }
                "booking_mode" => {
                    if let Some(v) = value.as_str() {
                        entity.booking_mode = v.to_string();
                    }
                }
                "status" => {
                    if let Some(v) = value.as_str() {
                        entity.status = v.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(
        &self,
        entity: TestTripPost,
        ctx: &mut FactoryContext,
    ) -> Result<TestTripPost> {
        // HTTP API mode
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "driver_id": entity.driver_id.to_string(),
                "origin_address": entity.origin_address,
                "origin_lat": entity.origin_lat,
                "origin_lng": entity.origin_lng,
                "destination_address": entity.destination_address,
                "destination_lat": entity.destination_lat,
                "destination_lng": entity.destination_lng,
                "departure_time": entity.departure_time.to_rfc3339(),
                "total_seats": entity.total_seats,
                "available_seats": entity.available_seats,
                "price_per_seat_cents": entity.price_per_seat_cents,
                "booking_mode": entity.booking_mode,
                "distance_km": entity.distance_km,
                "duration_minutes": entity.duration_minutes,
                "status": entity.status,
                "notes": entity.notes,
            });
            let resp = ctx.test_post("/__test__/trip-posts", &body).await?;
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
                INSERT INTO trip_posts (
                    id, driver_id, origin_address, origin_lat, origin_lng,
                    destination_address, destination_lat, destination_lng,
                    departure_time, total_seats, available_seats, price_per_seat_cents,
                    booking_mode, distance_km, duration_minutes, status, notes,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                "#,
            )
            .bind(entity.id)
            .bind(entity.driver_id)
            .bind(&entity.origin_address)
            .bind(entity.origin_lat)
            .bind(entity.origin_lng)
            .bind(&entity.destination_address)
            .bind(entity.destination_lat)
            .bind(entity.destination_lng)
            .bind(entity.departure_time)
            .bind(entity.total_seats)
            .bind(entity.available_seats)
            .bind(entity.price_per_seat_cents)
            .bind(&entity.booking_mode)
            .bind(entity.distance_km)
            .bind(entity.duration_minutes)
            .bind(&entity.status)
            .bind(&entity.notes)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- TripBookingFactory ---

pub struct TripBookingFactory {
    traits: TraitRegistry<TestTripBooking>,
}

impl TripBookingFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(PendingBookingTrait));
        traits.register(Box::new(ConfirmedBookingTrait));
        traits.register(Box::new(CompletedBookingTrait));
        traits.register(Box::new(CancelledBookingTrait));
        Self { traits }
    }
}

impl Default for TripBookingFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestTripBooking> for TripBookingFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestTripBooking {
        let now = Utc::now();
        let _n = ctx.sequence("booking");

        TestTripBooking {
            id: Uuid::new_v4(),
            trip_id: Uuid::new_v4(),
            passenger_id: Uuid::new_v4(),
            seats_booked: 1,
            total_price_cents: 1500,
            status: "confirmed".to_string(),
            pickup_point_address: None,
            pickup_point_lat: None,
            pickup_point_lng: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestTripBooking> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestTripBooking,
        overrides: &[(String, serde_json::Value)],
    ) {
        for (field, value) in overrides {
            match field.as_str() {
                "trip_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.trip_id = uuid;
                        }
                    }
                }
                "passenger_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.passenger_id = uuid;
                        }
                    }
                }
                "seats_booked" => {
                    if let Some(v) = value.as_i64() {
                        entity.seats_booked = v as i32;
                    }
                }
                "status" => {
                    if let Some(v) = value.as_str() {
                        entity.status = v.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(
        &self,
        entity: TestTripBooking,
        ctx: &mut FactoryContext,
    ) -> Result<TestTripBooking> {
        // HTTP API mode
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "trip_id": entity.trip_id.to_string(),
                "passenger_id": entity.passenger_id.to_string(),
                "seats_booked": entity.seats_booked,
                "total_price_cents": entity.total_price_cents,
                "status": entity.status,
                "pickup_point_address": entity.pickup_point_address,
                "pickup_point_lat": entity.pickup_point_lat,
                "pickup_point_lng": entity.pickup_point_lng,
            });
            let resp = ctx.test_post("/__test__/trip-bookings", &body).await?;
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
                INSERT INTO trip_bookings (
                    id, trip_id, passenger_id, seats_booked, total_price_cents,
                    status, pickup_point_address, pickup_point_lat, pickup_point_lng,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(entity.id)
            .bind(entity.trip_id)
            .bind(entity.passenger_id)
            .bind(entity.seats_booked)
            .bind(entity.total_price_cents)
            .bind(&entity.status)
            .bind(&entity.pickup_point_address)
            .bind(entity.pickup_point_lat)
            .bind(entity.pickup_point_lng)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- Trip Post Traits ---

struct InstantBookTrait;
impl FactoryTrait<TestTripPost> for InstantBookTrait {
    fn name(&self) -> &str {
        "instant_book"
    }
    fn apply(&self, trip: &mut TestTripPost) {
        trip.booking_mode = "instant".to_string();
    }
}

struct RequestToBookTrait;
impl FactoryTrait<TestTripPost> for RequestToBookTrait {
    fn name(&self) -> &str {
        "request_to_book"
    }
    fn apply(&self, trip: &mut TestTripPost) {
        trip.booking_mode = "approval".to_string();
    }
}

struct RecurringTripTrait;
impl FactoryTrait<TestTripPost> for RecurringTripTrait {
    fn name(&self) -> &str {
        "recurring"
    }
    fn apply(&self, trip: &mut TestTripPost) {
        trip.notes = Some("Recurring daily commute".to_string());
    }
}

struct WithStopsTrait;
impl FactoryTrait<TestTripPost> for WithStopsTrait {
    fn name(&self) -> &str {
        "with_stops"
    }
    fn apply(&self, trip: &mut TestTripPost) {
        trip.notes = Some("Stops at downtown transit center".to_string());
    }
}

struct FullTripTrait;
impl FactoryTrait<TestTripPost> for FullTripTrait {
    fn name(&self) -> &str {
        "full"
    }
    fn apply(&self, trip: &mut TestTripPost) {
        trip.available_seats = 0;
        trip.status = "full".to_string();
    }
}

// --- Trip Booking Traits ---

struct PendingBookingTrait;
impl FactoryTrait<TestTripBooking> for PendingBookingTrait {
    fn name(&self) -> &str {
        "pending"
    }
    fn apply(&self, booking: &mut TestTripBooking) {
        booking.status = "pending".to_string();
    }
}

struct ConfirmedBookingTrait;
impl FactoryTrait<TestTripBooking> for ConfirmedBookingTrait {
    fn name(&self) -> &str {
        "confirmed"
    }
    fn apply(&self, booking: &mut TestTripBooking) {
        booking.status = "confirmed".to_string();
    }
}

struct CompletedBookingTrait;
impl FactoryTrait<TestTripBooking> for CompletedBookingTrait {
    fn name(&self) -> &str {
        "completed"
    }
    fn apply(&self, booking: &mut TestTripBooking) {
        booking.status = "completed".to_string();
    }
}

struct CancelledBookingTrait;
impl FactoryTrait<TestTripBooking> for CancelledBookingTrait {
    fn name(&self) -> &str {
        "cancelled"
    }
    fn apply(&self, booking: &mut TestTripBooking) {
        booking.status = "cancelled".to_string();
    }
}
