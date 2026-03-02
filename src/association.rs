/// Describes a dependency between factory entities.
///
/// When a factory declares an association, the associated entity
/// is automatically created first (if not already provided via override).
///
/// For example, a `RideFactory` depends on a rider (user) and optionally
/// a driver. If no `rider_id` override is provided, the factory will
/// automatically create a user via `UserFactory` first.
#[derive(Debug, Clone)]
pub struct Association {
    /// The field name on the parent entity (e.g., "rider_id", "driver_id").
    pub field: String,

    /// The factory type name that produces the associated entity
    /// (e.g., "user", "driver_profile").
    pub factory_name: String,

    /// Whether this association is required (must be created) or optional.
    pub required: bool,

    /// Traits to apply to the associated entity when auto-creating.
    pub default_traits: Vec<String>,
}

impl Association {
    /// Create a required association.
    pub fn required(field: &str, factory_name: &str) -> Self {
        Self {
            field: field.to_string(),
            factory_name: factory_name.to_string(),
            required: true,
            default_traits: Vec::new(),
        }
    }

    /// Create an optional association.
    pub fn optional(field: &str, factory_name: &str) -> Self {
        Self {
            field: field.to_string(),
            factory_name: factory_name.to_string(),
            required: false,
            default_traits: Vec::new(),
        }
    }

    /// Set default traits to apply when auto-creating the association.
    pub fn with_traits(mut self, traits: &[&str]) -> Self {
        self.default_traits = traits.iter().map(|s| s.to_string()).collect();
        self
    }
}
