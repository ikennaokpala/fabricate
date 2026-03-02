use std::collections::HashMap;

/// A named modification that can be applied to a factory's output.
///
/// Traits are composable — multiple traits can be applied in sequence.
/// Later traits override earlier ones for the same fields.
pub trait FactoryTrait<T>: Send + Sync {
    /// The unique name of this trait (e.g., "verified", "with_vehicle").
    fn name(&self) -> &str;

    /// Apply the trait's modifications to the entity.
    fn apply(&self, entity: &mut T);
}

/// Registry of named traits for a specific entity type.
pub struct TraitRegistry<T> {
    traits: HashMap<String, Box<dyn FactoryTrait<T>>>,
}

impl<T> TraitRegistry<T> {
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
        }
    }

    /// Register a trait.
    pub fn register(&mut self, t: Box<dyn FactoryTrait<T>>) {
        self.traits.insert(t.name().to_string(), t);
    }

    /// Apply a named trait to an entity. Returns Err if trait not found.
    pub fn apply(&self, name: &str, entity: &mut T) -> crate::Result<()> {
        match self.traits.get(name) {
            Some(t) => {
                t.apply(entity);
                Ok(())
            }
            None => Err(crate::Error::TraitNotFound(format!(
                "Trait '{}' not registered. Available: {:?}",
                name,
                self.traits.keys().collect::<Vec<_>>()
            ))),
        }
    }

    /// Check if a trait is registered.
    pub fn has(&self, name: &str) -> bool {
        self.traits.contains_key(name)
    }

    /// List all registered trait names.
    pub fn names(&self) -> Vec<&str> {
        self.traits.keys().map(|s| s.as_str()).collect()
    }
}

impl<T> Default for TraitRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}
