use crate::builder::FactoryBuilder;
use crate::context::FactoryContext;
use crate::persona::*;
use crate::Result;

use super::driver::DriverProfileFactory;
use super::payment::{PaymentMethodFactory, WalletFactory};
use super::ride::RideFactory;
use super::trip::TripPostFactory;
use super::user::UserFactory;

/// Pre-built persona bundles for common test scenarios.
pub struct Personas;

impl Personas {
    /// Create a complete rider persona: user + wallet + payment_method.
    pub async fn rider(ctx: &mut FactoryContext) -> Result<RiderPersona> {
        let user = FactoryBuilder::new(UserFactory::new())
            .with_trait("verified")
            .create(ctx)
            .await?;

        let wallet = FactoryBuilder::new(WalletFactory::new())
            .set("user_id", serde_json::json!(user.id.to_string()))
            .with_trait("funded")
            .create(ctx)
            .await?;

        let pm = FactoryBuilder::new(PaymentMethodFactory::new())
            .set("user_id", serde_json::json!(user.id.to_string()))
            .create(ctx)
            .await?;

        Ok(RiderPersona {
            user_id: user.id,
            email: user.email,
            full_name: user.full_name,
            phone: user.phone_number,
            wallet_balance_cents: wallet.balance_cents,
            payment_method_card_last_four: pm.card_last_four,
            auth_token: None,
        })
    }

    /// Create a complete driver persona: user (driver type) + driver_profile + wallet.
    pub async fn driver(ctx: &mut FactoryContext) -> Result<DriverPersona> {
        let user = FactoryBuilder::new(UserFactory::new())
            .with_trait("verified")
            .with_trait("driver")
            .create(ctx)
            .await?;

        let dp = FactoryBuilder::new(DriverProfileFactory::new())
            .with_trait("verified")
            .set("user_id", serde_json::json!(user.id.to_string()))
            .create(ctx)
            .await?;

        let wallet = FactoryBuilder::new(WalletFactory::new())
            .set("user_id", serde_json::json!(user.id.to_string()))
            .create(ctx)
            .await?;

        Ok(DriverPersona {
            user_id: user.id,
            email: user.email,
            full_name: user.full_name,
            phone: user.phone_number,
            vehicle_make: dp.vehicle_make,
            vehicle_model: dp.vehicle_model,
            vehicle_year: dp.vehicle_year,
            vehicle_plate: dp.vehicle_plate_number,
            vehicle_color: dp.vehicle_color,
            average_rating: dp.average_rating,
            total_trips: dp.total_trips,
            is_verified: dp.is_license_verified && dp.is_insurance_verified,
            wallet_balance_cents: wallet.balance_cents,
            auth_token: None,
        })
    }

    /// Scenario: rider books a ride from a driver.
    pub async fn rider_books_ride(ctx: &mut FactoryContext) -> Result<BookingScenario> {
        let rider = Self::rider(ctx).await?;
        let driver = Self::driver(ctx).await?;

        let ride = FactoryBuilder::new(RideFactory::new())
            .with_trait("accepted")
            .set("passenger_id", serde_json::json!(rider.user_id.to_string()))
            .set("driver_id", serde_json::json!(driver.user_id.to_string()))
            .create(ctx)
            .await?;

        Ok(BookingScenario {
            rider,
            driver,
            ride_id: ride.id,
            ride_status: ride.status,
            pickup_address: ride.pickup_address,
            destination_address: ride.destination_address,
            estimated_price_cents: ride.estimated_price_cents,
        })
    }

    /// Scenario: driver onboard (unverified, needs to complete setup).
    pub async fn driver_onboard(ctx: &mut FactoryContext) -> Result<DriverPersona> {
        let user = FactoryBuilder::new(UserFactory::new())
            .with_trait("driver")
            .create(ctx)
            .await?;

        let dp = FactoryBuilder::new(DriverProfileFactory::new())
            .with_trait("unverified")
            .set("user_id", serde_json::json!(user.id.to_string()))
            .create(ctx)
            .await?;

        Ok(DriverPersona {
            user_id: user.id,
            email: user.email,
            full_name: user.full_name,
            phone: user.phone_number,
            vehicle_make: dp.vehicle_make,
            vehicle_model: dp.vehicle_model,
            vehicle_year: dp.vehicle_year,
            vehicle_plate: dp.vehicle_plate_number,
            vehicle_color: dp.vehicle_color,
            average_rating: dp.average_rating,
            total_trips: dp.total_trips,
            is_verified: false,
            wallet_balance_cents: 0,
            auth_token: None,
        })
    }

    /// Scenario: completed ride with payment.
    pub async fn complete_ride(ctx: &mut FactoryContext) -> Result<CompletedRideScenario> {
        let rider = Self::rider(ctx).await?;
        let driver = Self::driver(ctx).await?;

        let ride = FactoryBuilder::new(RideFactory::new())
            .with_trait("completed")
            .with_trait("with_payment")
            .with_trait("with_ratings")
            .set("passenger_id", serde_json::json!(rider.user_id.to_string()))
            .set("driver_id", serde_json::json!(driver.user_id.to_string()))
            .create(ctx)
            .await?;

        Ok(CompletedRideScenario {
            rider,
            driver,
            ride_id: ride.id,
            final_price_cents: ride.final_price_cents.unwrap_or(ride.estimated_price_cents),
            payment_status: ride.payment_status,
            driver_rating: ride.driver_rating,
            rider_rating: ride.passenger_rating,
        })
    }

    /// Scenario: driver posts a trip (carpooling/Poparide).
    pub async fn driver_posts_trip(ctx: &mut FactoryContext) -> Result<TripPostScenario> {
        let driver = Self::driver(ctx).await?;

        let trip = FactoryBuilder::new(TripPostFactory::new())
            .with_trait("instant_book")
            .set("driver_id", serde_json::json!(driver.user_id.to_string()))
            .create(ctx)
            .await?;

        Ok(TripPostScenario {
            driver,
            trip_id: trip.id,
            origin_address: trip.origin_address,
            destination_address: trip.destination_address,
            available_seats: trip.available_seats,
            price_per_seat_cents: trip.price_per_seat_cents,
            booking_mode: trip.booking_mode,
        })
    }

    /// Seed a comprehensive set of data for Sentinel exploration.
    pub async fn seed_for_exploration(ctx: &mut FactoryContext) -> Result<SeedSummary> {
        let mut riders = Vec::new();
        let mut drivers = Vec::new();
        let mut bookings = Vec::new();
        let mut completed_rides = Vec::new();
        let mut trip_posts = Vec::new();
        let mut total = 0;

        // 3 riders
        for _ in 0..3 {
            riders.push(Self::rider(ctx).await?);
            total += 3; // user + wallet + payment_method
        }

        // 3 drivers (verified)
        for _ in 0..3 {
            drivers.push(Self::driver(ctx).await?);
            total += 3; // user + driver_profile + wallet
        }

        // 1 onboarding driver
        drivers.push(Self::driver_onboard(ctx).await?);
        total += 2; // user + driver_profile

        // 2 active bookings (rider books ride)
        for _ in 0..2 {
            bookings.push(Self::rider_books_ride(ctx).await?);
            total += 7; // rider(3) + driver(3) + ride(1)
        }

        // 3 completed rides (with payment + ratings)
        for _ in 0..3 {
            completed_rides.push(Self::complete_ride(ctx).await?);
            total += 7;
        }

        // 2 trip posts
        for _ in 0..2 {
            trip_posts.push(Self::driver_posts_trip(ctx).await?);
            total += 4; // driver(3) + trip(1)
        }

        Ok(SeedSummary {
            riders,
            drivers,
            bookings,
            completed_rides,
            trip_posts,
            total_entities_created: total,
        })
    }

    /// Seed data via the backend's `/__test__/seed` API.
    ///
    /// Calls the backend's seed endpoint directly with `{ seed, scenario }`.
    /// The backend handles all entity creation internally and returns IDs.
    pub async fn seed_via_api(
        ctx: &mut FactoryContext,
        scenarios: &[&str],
    ) -> Result<SeedSummary> {
        let mut riders = Vec::new();
        let mut drivers = Vec::new();
        let mut bookings = Vec::new();
        let mut completed_rides = Vec::new();
        let trip_posts = Vec::new();
        let mut total = 0;

        // Map user-facing scenario names to backend Scenario enum values
        let scenario_map: &[(&str, &str)] = &[
            ("rider-books-ride", "rider_book_ride"),
            ("driver-onboard", "driver_onboard"),
            ("complete-ride", "complete_ride"),
            ("payment-flow", "payment_flow"),
            ("sos-emergency", "sos_emergency"),
        ];

        let resolve_scenario = |name: &str| -> Option<&str> {
            scenario_map
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| *v)
        };

        // "full" seeds all 5 backend scenarios
        let effective: Vec<&str> = if scenarios.contains(&"full") {
            scenario_map.iter().map(|(_, v)| *v).collect()
        } else {
            scenarios
                .iter()
                .map(|s| resolve_scenario(s).unwrap_or(s))
                .collect()
        };

        for backend_scenario in &effective {
            let seed = ctx.sequence("seed");
            let body = serde_json::json!({
                "seed": seed,
                "scenario": backend_scenario,
            });
            let resp = ctx.test_post("/__test__/seed", &body).await?;

            // Parse the backend SeedResponse.entities into our persona types
            let entities = resp.get("entities").cloned().unwrap_or_default();

            if let Some(rider_id) = entities.get("rider_id").and_then(|v| v.as_str()) {
                if let Ok(uid) = uuid::Uuid::parse_str(rider_id) {
                    riders.push(RiderPersona {
                        user_id: uid,
                        email: entities.get("rider_email").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        full_name: "Test Rider".to_string(),
                        phone: String::new(),
                        wallet_balance_cents: 0,
                        payment_method_card_last_four: String::new(),
                        auth_token: entities.get("auth_token").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    });
                    total += 1;
                }
            }

            if let Some(driver_id) = entities.get("driver_id").and_then(|v| v.as_str()) {
                if let Ok(uid) = uuid::Uuid::parse_str(driver_id) {
                    drivers.push(DriverPersona {
                        user_id: uid,
                        email: entities.get("driver_email").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        full_name: "Test Driver".to_string(),
                        phone: String::new(),
                        vehicle_make: "Toyota".to_string(),
                        vehicle_model: "Camry".to_string(),
                        vehicle_year: 2024,
                        vehicle_plate: String::new(),
                        vehicle_color: String::new(),
                        average_rating: 4.8,
                        total_trips: 0,
                        is_verified: true,
                        wallet_balance_cents: 0,
                        auth_token: None,
                    });
                    total += 1;
                }
            }

            if let Some(ride_id) = entities.get("ride_id").and_then(|v| v.as_str()) {
                if let Ok(rid) = uuid::Uuid::parse_str(ride_id) {
                    // Determine which collection based on scenario
                    match *backend_scenario {
                        "rider_book_ride" => {
                            if let (Some(rider), Some(driver)) = (riders.last().cloned(), drivers.last().cloned()) {
                                bookings.push(BookingScenario {
                                    rider,
                                    driver,
                                    ride_id: rid,
                                    ride_status: "requested".to_string(),
                                    pickup_address: "Test Pickup".to_string(),
                                    destination_address: "Test Destination".to_string(),
                                    estimated_price_cents: 2500,
                                });
                            }
                        }
                        "complete_ride" => {
                            if let (Some(rider), Some(driver)) = (riders.last().cloned(), drivers.last().cloned()) {
                                completed_rides.push(CompletedRideScenario {
                                    rider,
                                    driver,
                                    ride_id: rid,
                                    final_price_cents: 2500,
                                    payment_status: "completed".to_string(),
                                    driver_rating: Some(5),
                                    rider_rating: Some(5),
                                });
                            }
                        }
                        _ => {}
                    }
                    total += 1;
                }
            }
        }

        Ok(SeedSummary {
            riders,
            drivers,
            bookings,
            completed_rides,
            trip_posts,
            total_entities_created: total,
        })
    }
}
