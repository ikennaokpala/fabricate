use async_trait::async_trait;

use crate::context::FactoryContext;
use crate::Result;

/// Core factory trait for building and persisting test entities.
///
/// Implement this for each entity type you want to generate test data for.
/// The `build` method creates in-memory instances, while `create` persists
/// them to the database.
#[async_trait]
pub trait Factory: Send + Sync {
    /// The output entity type produced by this factory.
    type Output: Send;

    /// Build an in-memory instance without persisting to the database.
    fn build(&self, ctx: &mut FactoryContext) -> Self::Output;

    /// Persist an entity to the database and return the created record.
    async fn create(&self, ctx: &mut FactoryContext) -> Result<Self::Output>;

    /// Create multiple entities at once.
    async fn create_list(&self, ctx: &mut FactoryContext, count: usize) -> Result<Vec<Self::Output>> {
        let mut results = Vec::with_capacity(count);
        for _ in 0..count {
            results.push(self.create(ctx).await?);
        }
        Ok(results)
    }
}

/// Strategy for how a factory produces its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStrategy {
    /// In-memory only, no database interaction.
    Build,
    /// Persist to database via sqlx.
    Create,
    /// Create via HTTP API (e.g., `/__test__/seed` endpoint).
    HttpCreate,
}
