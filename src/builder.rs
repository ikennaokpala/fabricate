use crate::context::FactoryContext;
use crate::traits::TraitRegistry;
use crate::Result;

/// Fluent builder for constructing factory entities with traits and overrides.
///
/// ```ignore
/// let user = FactoryBuilder::new(UserFactory::default())
///     .with_trait("verified")
///     .set("email", json!("custom@test.com"))
///     .build(&mut ctx);
/// ```
pub struct FactoryBuilder<F, T> {
    factory: F,
    trait_names: Vec<String>,
    overrides: Vec<(String, serde_json::Value)>,
    _phantom: std::marker::PhantomData<T>,
}

impl<F, T> FactoryBuilder<F, T>
where
    F: BuildableFactory<T>,
{
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            trait_names: Vec::new(),
            overrides: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Apply a named trait to the entity.
    pub fn with_trait(mut self, name: &str) -> Self {
        self.trait_names.push(name.to_string());
        self
    }

    /// Override a specific field value.
    pub fn set(mut self, field: &str, value: serde_json::Value) -> Self {
        self.overrides.push((field.to_string(), value));
        self
    }

    /// Build the entity in memory (no DB).
    pub fn build(self, ctx: &mut FactoryContext) -> Result<T> {
        // Apply overrides to context
        for (field, value) in &self.overrides {
            ctx.set_override(field, value.clone());
        }

        // Build base entity
        let mut entity = self.factory.build_base(ctx);

        // Apply traits
        let registry = self.factory.trait_registry();
        for name in &self.trait_names {
            registry.apply(name, &mut entity)?;
        }

        // Apply field-level overrides
        self.factory.apply_overrides(&mut entity, &self.overrides);

        // Clear overrides from context
        ctx.clear_overrides();

        Ok(entity)
    }

    /// Create the entity, persisting to database.
    pub async fn create(self, ctx: &mut FactoryContext) -> Result<T> {
        // Apply overrides to context
        for (field, value) in &self.overrides {
            ctx.set_override(field, value.clone());
        }

        // Build and create via factory
        let mut entity = self.factory.build_base(ctx);

        // Apply traits
        let registry = self.factory.trait_registry();
        for name in &self.trait_names {
            registry.apply(name, &mut entity)?;
        }

        // Apply field-level overrides
        self.factory.apply_overrides(&mut entity, &self.overrides);

        // Persist
        let result = self.factory.persist(entity, ctx).await?;

        // Clear overrides
        ctx.clear_overrides();

        Ok(result)
    }
}

/// Trait that factories must implement to work with the builder.
pub trait BuildableFactory<T>: Send + Sync {
    /// Build the base entity with default values.
    fn build_base(&self, ctx: &mut FactoryContext) -> T;

    /// Get the trait registry for this factory.
    fn trait_registry(&self) -> &TraitRegistry<T>;

    /// Apply field-level overrides from `.set()` calls.
    fn apply_overrides(&self, entity: &mut T, overrides: &[(String, serde_json::Value)]);

    /// Persist the entity to the database or API.
    fn persist(
        &self,
        entity: T,
        ctx: &mut FactoryContext,
    ) -> impl std::future::Future<Output = Result<T>> + Send;
}
