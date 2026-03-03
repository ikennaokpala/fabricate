use clap::{Parser, Subcommand};
use fabricate::context::FactoryContext;
use fabricate::ridemate::personas::Personas;

#[derive(Parser)]
#[command(name = "fabricate", version, about = "FactoryBot-inspired test data factory for Rust + sqlx")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Seed test data via backend API or direct DB
    Seed {
        /// Backend API base URL (e.g., http://localhost:8080)
        #[arg(long, default_value = "http://localhost:8080")]
        target: String,

        /// Comma-separated persona names to seed (rider, driver)
        #[arg(long)]
        personas: Option<String>,

        /// Comma-separated scenario names (rider-books-ride, complete-ride, driver-posts-trip, full)
        #[arg(long)]
        scenarios: Option<String>,

        /// Test API key for /__test__/ endpoints
        #[arg(long, default_value = "test-key")]
        test_key: String,

        /// Direct database URL (bypasses HTTP API)
        #[arg(long)]
        database_url: Option<String>,
    },

    /// Reset (delete) all test data
    Reset {
        /// Backend API base URL
        #[arg(long, default_value = "http://localhost:8080")]
        target: String,

        /// Scope of reset: "all" or specific entity type
        #[arg(long, default_value = "all")]
        scope: String,

        /// Test API key
        #[arg(long, default_value = "test-key")]
        test_key: String,
    },

    /// List available factories, traits, and personas
    List,

    /// Check backend health / connectivity
    Health {
        /// Backend API base URL (e.g., http://localhost:8080)
        #[arg(long, default_value = "http://localhost:8080")]
        target: String,

        /// Test API key for /__test__/ endpoints
        #[arg(long, default_value = "test-key")]
        test_key: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Seed {
            target,
            personas,
            scenarios,
            test_key,
            database_url,
        } => {
            run_seed(target, personas, scenarios, test_key, database_url).await?;
        }
        Commands::Reset {
            target,
            scope,
            test_key,
        } => {
            run_reset(target, scope, test_key).await?;
        }
        Commands::List => {
            run_list();
        }
        Commands::Health { target, test_key } => {
            run_health(target, test_key).await?;
        }
    }

    Ok(())
}

async fn run_seed(
    target: String,
    personas: Option<String>,
    scenarios: Option<String>,
    test_key: String,
    _database_url: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("fabricate: Seeding test data...");
    println!("  Target: {target}");

    let mut ctx = FactoryContext::http(&target).with_test_key(&test_key);

    // Determine what to seed
    let scenario_list: Vec<&str> = if let Some(ref s) = scenarios {
        s.split(',').map(|s| s.trim()).collect()
    } else if let Some(ref p) = personas {
        p.split(',').map(|s| s.trim()).collect()
    } else {
        vec!["full"]
    };

    println!("  Scenarios: {}", scenario_list.join(", "));

    let summary = Personas::seed_via_api(&mut ctx, &scenario_list).await?;

    println!("\nSeed Summary:");
    println!("  Riders:          {}", summary.riders.len());
    println!("  Drivers:         {}", summary.drivers.len());
    println!("  Bookings:        {}", summary.bookings.len());
    println!("  Completed Rides: {}", summary.completed_rides.len());
    println!("  Trip Posts:      {}", summary.trip_posts.len());
    println!(
        "  Total Entities:  {}",
        summary.total_entities_created
    );

    // Print details
    for (i, rider) in summary.riders.iter().enumerate() {
        println!("\n  Rider {}: {} <{}>", i + 1, rider.full_name, rider.email);
    }
    for (i, driver) in summary.drivers.iter().enumerate() {
        println!(
            "\n  Driver {}: {} <{}> - {} {} {}",
            i + 1,
            driver.full_name,
            driver.email,
            driver.vehicle_year,
            driver.vehicle_make,
            driver.vehicle_model
        );
    }

    println!("\nSeed complete.");
    Ok(())
}

async fn run_reset(
    target: String,
    scope: String,
    test_key: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("fabricate: Resetting test data...");
    println!("  Target: {target}");
    println!("  Scope: {scope}");

    let ctx = FactoryContext::http(&target).with_test_key(&test_key);

    let body = serde_json::json!({ "scope": scope });
    ctx.test_post("/__test__/reset", &body).await?;

    println!("Reset complete.");
    Ok(())
}

async fn run_health(
    target: String,
    test_key: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("fabricate: Checking backend health...");
    println!("  Target: {target}");

    let ctx = FactoryContext::http(&target).with_test_key(&test_key);
    let healthy = ctx.health_check().await?;

    if healthy {
        println!("  Status: OK");
    } else {
        println!("  Status: UNHEALTHY");
        std::process::exit(1);
    }

    Ok(())
}

fn run_list() {
    println!("fabricate: Available Factories & Personas\n");

    println!("FACTORIES:");
    println!("  UserFactory");
    println!("    Traits: verified, unverified, suspended, premium, driver, admin");
    println!("  DriverProfileFactory");
    println!("    Traits: verified, unverified, high_rated, new_driver, available, offline");
    println!("  RideFactory");
    println!("    Traits: requested, accepted, in_progress, completed, cancelled, with_payment, with_ratings");
    println!("  TripPostFactory");
    println!("    Traits: instant_book, request_to_book, recurring, with_stops, full");
    println!("  TripBookingFactory");
    println!("    Traits: pending, confirmed, completed, cancelled");
    println!("  PaymentMethodFactory");
    println!("    Traits: visa, mastercard");
    println!("  WalletFactory");
    println!("    Traits: funded, empty");
    println!("  PaymentFactory");
    println!("    Traits: successful, failed, refunded, pending");
    println!("  RatingFactory");
    println!("    Traits: five_star, low_rating, with_review, driver_rating, passenger_rating");
    println!("  SafetyIncidentFactory");
    println!("    Traits: panic_button, crash_detected, resolved");
    println!("  SafetyContactFactory");
    println!("    Traits: (none)");

    println!("\nPERSONAS (scenario bundles):");
    println!("  rider                  - User + wallet + payment method");
    println!("  driver                 - User (driver) + driver profile + wallet");
    println!("  driver-onboard         - Driver (unverified, needs setup)");
    println!("  rider-books-ride       - Rider + driver + accepted ride");
    println!("  complete-ride          - Rider + driver + completed ride + payment + ratings");
    println!("  driver-posts-trip      - Driver + trip post (carpooling)");
    println!("  full                   - All of the above (comprehensive exploration data)");
}
