pub mod driver;
pub mod payment;
pub mod personas;
pub mod rating;
pub mod ride;
pub mod safety;
pub mod trip;
pub mod user;

pub use driver::DriverProfileFactory;
pub use payment::{PaymentFactory, PaymentMethodFactory, WalletFactory};
pub use personas::Personas;
pub use rating::RatingFactory;
pub use ride::RideFactory;
pub use safety::{SafetyContactFactory, SafetyIncidentFactory};
pub use trip::{TripBookingFactory, TripPostFactory};
pub use user::UserFactory;

/// SF Bay Area location range for generating test coordinates.
pub const SF_LAT_MIN: f64 = 37.70;
pub const SF_LAT_MAX: f64 = 37.85;
pub const SF_LNG_MIN: f64 = -122.50;
pub const SF_LNG_MAX: f64 = -122.35;

/// Generate a deterministic SF Bay Area latitude from a sequence number.
pub fn test_latitude(n: u64) -> f64 {
    SF_LAT_MIN + ((n as f64 * 0.0137) % (SF_LAT_MAX - SF_LAT_MIN))
}

/// Generate a deterministic SF Bay Area longitude from a sequence number.
pub fn test_longitude(n: u64) -> f64 {
    SF_LNG_MIN + ((n as f64 * 0.0113) % (SF_LNG_MAX - SF_LNG_MIN))
}

/// Generate a deterministic SF street address from a sequence number.
pub fn test_address(n: u64) -> String {
    let streets = [
        "Market St",
        "Mission St",
        "Valencia St",
        "Folsom St",
        "Howard St",
        "Harrison St",
        "Bryant St",
        "Brannan St",
        "Townsend St",
        "King St",
    ];
    let num = 100 + (n * 17) % 900;
    let street = streets[(n as usize) % streets.len()];
    format!("{num} {street}, San Francisco, CA")
}

/// Vehicle make/model pairs.
pub const VEHICLES: &[(&str, &str)] = &[
    ("Toyota", "Camry"),
    ("Honda", "Civic"),
    ("Tesla", "Model 3"),
    ("Ford", "Escape"),
    ("Chevrolet", "Malibu"),
    ("Hyundai", "Sonata"),
    ("Nissan", "Altima"),
    ("Kia", "Optima"),
    ("Subaru", "Outback"),
    ("Mazda", "CX-5"),
];

/// Vehicle colors.
pub const COLORS: &[&str] = &[
    "Black", "White", "Silver", "Blue", "Red", "Gray", "Green", "Brown",
];
