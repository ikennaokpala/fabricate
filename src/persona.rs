use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A complete rider persona with all associated entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiderPersona {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub phone: String,
    pub wallet_balance_cents: i64,
    pub payment_method_card_last_four: String,
    pub auth_token: Option<String>,
}

/// A complete driver persona with all associated entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverPersona {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub phone: String,
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub vehicle_year: i32,
    pub vehicle_plate: String,
    pub vehicle_color: String,
    pub average_rating: f64,
    pub total_trips: i32,
    pub is_verified: bool,
    pub wallet_balance_cents: i64,
    pub auth_token: Option<String>,
}

/// A complete booking scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingScenario {
    pub rider: RiderPersona,
    pub driver: DriverPersona,
    pub ride_id: Uuid,
    pub ride_status: String,
    pub pickup_address: String,
    pub destination_address: String,
    pub estimated_price_cents: i64,
}

/// A completed ride scenario (with payment and ratings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedRideScenario {
    pub rider: RiderPersona,
    pub driver: DriverPersona,
    pub ride_id: Uuid,
    pub final_price_cents: i64,
    pub payment_status: String,
    pub driver_rating: Option<i32>,
    pub rider_rating: Option<i32>,
}

/// A trip post scenario (carpooling/Poparide).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripPostScenario {
    pub driver: DriverPersona,
    pub trip_id: Uuid,
    pub origin_address: String,
    pub destination_address: String,
    pub available_seats: i32,
    pub price_per_seat_cents: i64,
    pub booking_mode: String,
}

/// Summary of all seeded data for a full exploration session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedSummary {
    pub riders: Vec<RiderPersona>,
    pub drivers: Vec<DriverPersona>,
    pub bookings: Vec<BookingScenario>,
    pub completed_rides: Vec<CompletedRideScenario>,
    pub trip_posts: Vec<TripPostScenario>,
    pub total_entities_created: usize,
}
