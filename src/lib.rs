pub mod association;
pub mod builder;
pub mod context;
pub mod factory;
pub mod persona;
pub mod sequence;
pub mod traits;

pub mod ridemate;

pub use builder::FactoryBuilder;
pub use context::FactoryContext;
pub use factory::Factory;
pub use sequence::Sequence;
pub use traits::FactoryTrait;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Factory build error: {0}")]
    Build(String),

    #[error("Association error: {0}")]
    Association(String),

    #[error("Trait not found: {0}")]
    TraitNotFound(String),

    #[error("Sequence error: {0}")]
    Sequence(String),

    #[cfg(feature = "postgres")]
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Persona error: {0}")]
    Persona(String),
}
