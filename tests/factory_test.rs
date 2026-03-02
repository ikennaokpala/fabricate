use fabricate::builder::FactoryBuilder;
use fabricate::context::FactoryContext;
use fabricate::ridemate::driver::DriverProfileFactory;
use fabricate::ridemate::payment::{PaymentMethodFactory, WalletFactory};
use fabricate::ridemate::rating::RatingFactory;
use fabricate::ridemate::ride::RideFactory;
use fabricate::ridemate::safety::{SafetyContactFactory, SafetyIncidentFactory};
use fabricate::ridemate::trip::{TripBookingFactory, TripPostFactory};
use fabricate::ridemate::user::UserFactory;

fn make_ctx() -> FactoryContext {
    // No DB, no HTTP — build-only mode
    FactoryContext {
        sequences: fabricate::Sequence::new(),
        pool: None,
        http_client: None,
        base_url: None,
        test_key: "test-key".to_string(),
        overrides: std::collections::HashMap::new(),
    }
}

#[test]
fn test_user_factory_build() {
    let mut ctx = make_ctx();
    let user = FactoryBuilder::new(UserFactory::new())
        .build(&mut ctx)
        .unwrap();

    assert!(user.email.starts_with("test_user_"));
    assert!(user.email.ends_with("@test.ridemate.app"));
    assert!(user.is_email_verified);
    assert_eq!(user.user_type, "passenger");
    assert_eq!(user.role, "user");
}

#[test]
fn test_user_factory_with_traits() {
    let mut ctx = make_ctx();

    // Verified + driver
    let driver = FactoryBuilder::new(UserFactory::new())
        .with_trait("verified")
        .with_trait("driver")
        .build(&mut ctx)
        .unwrap();

    assert!(driver.is_email_verified);
    assert!(driver.is_phone_verified);
    assert_eq!(driver.user_type, "driver");

    // Suspended
    let suspended = FactoryBuilder::new(UserFactory::new())
        .with_trait("suspended")
        .build(&mut ctx)
        .unwrap();

    assert!(!suspended.is_active);
}

#[test]
fn test_user_factory_with_overrides() {
    let mut ctx = make_ctx();
    let user = FactoryBuilder::new(UserFactory::new())
        .set("email", serde_json::json!("custom@test.com"))
        .set("full_name", serde_json::json!("Custom Name"))
        .build(&mut ctx)
        .unwrap();

    assert_eq!(user.email, "custom@test.com");
    assert_eq!(user.full_name, "Custom Name");
}

#[test]
fn test_driver_profile_factory() {
    let mut ctx = make_ctx();
    let dp = FactoryBuilder::new(DriverProfileFactory::new())
        .with_trait("verified")
        .with_trait("high_rated")
        .build(&mut ctx)
        .unwrap();

    assert!(dp.is_license_verified);
    assert!(dp.is_insurance_verified);
    assert_eq!(dp.background_check_status, "completed");
    assert!(dp.average_rating >= 4.8);
    assert_eq!(dp.total_trips, 250);
}

#[test]
fn test_ride_factory_statuses() {
    let mut ctx = make_ctx();

    let requested = FactoryBuilder::new(RideFactory::new())
        .with_trait("requested")
        .build(&mut ctx)
        .unwrap();
    assert_eq!(requested.status, "requested");
    assert!(requested.driver_id.is_none());

    let completed = FactoryBuilder::new(RideFactory::new())
        .with_trait("completed")
        .build(&mut ctx)
        .unwrap();
    assert_eq!(completed.status, "completed");
    assert!(completed.completed_at.is_some());
    assert!(completed.final_price_cents.is_some());
    assert_eq!(completed.payment_status, "succeeded");

    let cancelled = FactoryBuilder::new(RideFactory::new())
        .with_trait("cancelled")
        .build(&mut ctx)
        .unwrap();
    assert_eq!(cancelled.status, "cancelled");
    assert!(cancelled.cancelled_at.is_some());
}

#[test]
fn test_trip_post_factory() {
    let mut ctx = make_ctx();
    let trip = FactoryBuilder::new(TripPostFactory::new())
        .with_trait("instant_book")
        .build(&mut ctx)
        .unwrap();

    assert_eq!(trip.booking_mode, "instant");
    assert_eq!(trip.total_seats, 3);
    assert_eq!(trip.available_seats, 3);
    assert_eq!(trip.status, "upcoming");
}

#[test]
fn test_trip_booking_factory() {
    let mut ctx = make_ctx();
    let booking = FactoryBuilder::new(TripBookingFactory::new())
        .with_trait("pending")
        .build(&mut ctx)
        .unwrap();

    assert_eq!(booking.status, "pending");
    assert_eq!(booking.seats_booked, 1);
}

#[test]
fn test_payment_method_factory() {
    let mut ctx = make_ctx();
    let pm = FactoryBuilder::new(PaymentMethodFactory::new())
        .with_trait("mastercard")
        .build(&mut ctx)
        .unwrap();

    assert_eq!(pm.card_brand, "mastercard");
    assert_eq!(pm.card_last_four, "5555");
    assert!(pm.is_default);
}

#[test]
fn test_wallet_factory() {
    let mut ctx = make_ctx();
    let wallet = FactoryBuilder::new(WalletFactory::new())
        .with_trait("funded")
        .build(&mut ctx)
        .unwrap();

    assert_eq!(wallet.balance_cents, 10000);
    assert!(wallet.is_verified);
}

#[test]
fn test_rating_factory() {
    let mut ctx = make_ctx();
    let rating = FactoryBuilder::new(RatingFactory::new())
        .with_trait("five_star")
        .with_trait("with_review")
        .build(&mut ctx)
        .unwrap();

    assert_eq!(rating.overall_score, 5);
    assert!(rating.review_text.is_some());
}

#[test]
fn test_safety_incident_factory() {
    let mut ctx = make_ctx();
    let incident = FactoryBuilder::new(SafetyIncidentFactory::new())
        .with_trait("panic_button")
        .build(&mut ctx)
        .unwrap();

    assert_eq!(incident.incident_type, "panic_button");
    assert_eq!(incident.severity, 10);
    assert_eq!(incident.status, "active");
}

#[test]
fn test_safety_contact_factory() {
    let mut ctx = make_ctx();
    let contact = FactoryBuilder::new(SafetyContactFactory::new())
        .build(&mut ctx)
        .unwrap();

    assert!(contact.name.starts_with("Emergency Contact"));
    assert_eq!(contact.relationship, "family");
    assert!(contact.is_primary);
}

#[test]
fn test_sequence_uniqueness() {
    let mut ctx = make_ctx();

    let u1 = FactoryBuilder::new(UserFactory::new()).build(&mut ctx).unwrap();
    let u2 = FactoryBuilder::new(UserFactory::new()).build(&mut ctx).unwrap();

    assert_ne!(u1.email, u2.email);
    assert_ne!(u1.phone_number, u2.phone_number);
    assert_ne!(u1.id, u2.id);
}

#[test]
fn test_invalid_trait_returns_error() {
    let mut ctx = make_ctx();
    let result = FactoryBuilder::new(UserFactory::new())
        .with_trait("nonexistent_trait")
        .build(&mut ctx);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("nonexistent_trait"));
}
