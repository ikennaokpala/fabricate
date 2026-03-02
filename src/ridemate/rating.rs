use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::builder::BuildableFactory;
use crate::context::FactoryContext;
use crate::traits::{FactoryTrait, TraitRegistry};
use crate::Result;

/// Rating entity matching the backend `ratings` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRating {
    pub id: Uuid,
    pub ride_id: Uuid,
    pub rater_id: Uuid,
    pub ratee_id: Uuid,
    pub role_rated: String,
    pub overall_score: i16,
    pub safety_score: Option<i16>,
    pub cleanliness_score: Option<i16>,
    pub communication_score: Option<i16>,
    pub timeliness_score: Option<i16>,
    pub review_text: Option<String>,
    pub verified: bool,
    pub flagged: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct RatingFactory {
    traits: TraitRegistry<TestRating>,
}

impl RatingFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(FiveStarTrait));
        traits.register(Box::new(LowRatingTrait));
        traits.register(Box::new(WithReviewTrait));
        traits.register(Box::new(DriverRatingTrait));
        traits.register(Box::new(PassengerRatingTrait));
        Self { traits }
    }
}

impl Default for RatingFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestRating> for RatingFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestRating {
        let _n = ctx.sequence("rating");
        let now = Utc::now();

        TestRating {
            id: Uuid::new_v4(),
            ride_id: Uuid::new_v4(),
            rater_id: Uuid::new_v4(),
            ratee_id: Uuid::new_v4(),
            role_rated: "driver".to_string(),
            overall_score: 5,
            safety_score: Some(5),
            cleanliness_score: Some(5),
            communication_score: Some(5),
            timeliness_score: Some(5),
            review_text: None,
            verified: true,
            flagged: false,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestRating> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestRating,
        overrides: &[(String, serde_json::Value)],
    ) {
        for (field, value) in overrides {
            match field.as_str() {
                "ride_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.ride_id = uuid;
                        }
                    }
                }
                "rater_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.rater_id = uuid;
                        }
                    }
                }
                "ratee_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.ratee_id = uuid;
                        }
                    }
                }
                "overall_score" => {
                    if let Some(v) = value.as_i64() {
                        entity.overall_score = v as i16;
                    }
                }
                "role_rated" => {
                    if let Some(v) = value.as_str() {
                        entity.role_rated = v.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(&self, entity: TestRating, ctx: &mut FactoryContext) -> Result<TestRating> {
        // HTTP API mode
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "ride_id": entity.ride_id.to_string(),
                "rater_id": entity.rater_id.to_string(),
                "ratee_id": entity.ratee_id.to_string(),
                "role_rated": entity.role_rated,
                "overall_score": entity.overall_score,
                "safety_score": entity.safety_score,
                "cleanliness_score": entity.cleanliness_score,
                "communication_score": entity.communication_score,
                "timeliness_score": entity.timeliness_score,
                "review_text": entity.review_text,
            });
            let resp = ctx.test_post("/__test__/ratings", &body).await?;
            let mut result = entity.clone();
            if let Some(id) = resp.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = Uuid::parse_str(id) {
                    result.id = uuid;
                }
            }
            return Ok(result);
        }

        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            sqlx::query(
                r#"
                INSERT INTO ratings (
                    id, ride_id, rater_id, ratee_id, role_rated, overall_score,
                    safety_score, cleanliness_score, communication_score, timeliness_score,
                    review_text, verified, flagged, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5::rated_role, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                "#,
            )
            .bind(entity.id)
            .bind(entity.ride_id)
            .bind(entity.rater_id)
            .bind(entity.ratee_id)
            .bind(&entity.role_rated)
            .bind(entity.overall_score)
            .bind(entity.safety_score)
            .bind(entity.cleanliness_score)
            .bind(entity.communication_score)
            .bind(entity.timeliness_score)
            .bind(&entity.review_text)
            .bind(entity.verified)
            .bind(entity.flagged)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- Traits ---

struct FiveStarTrait;
impl FactoryTrait<TestRating> for FiveStarTrait {
    fn name(&self) -> &str {
        "five_star"
    }
    fn apply(&self, rating: &mut TestRating) {
        rating.overall_score = 5;
        rating.safety_score = Some(5);
        rating.cleanliness_score = Some(5);
        rating.communication_score = Some(5);
        rating.timeliness_score = Some(5);
    }
}

struct LowRatingTrait;
impl FactoryTrait<TestRating> for LowRatingTrait {
    fn name(&self) -> &str {
        "low_rating"
    }
    fn apply(&self, rating: &mut TestRating) {
        rating.overall_score = 2;
        rating.safety_score = Some(3);
        rating.cleanliness_score = Some(2);
        rating.communication_score = Some(2);
        rating.timeliness_score = Some(1);
    }
}

struct WithReviewTrait;
impl FactoryTrait<TestRating> for WithReviewTrait {
    fn name(&self) -> &str {
        "with_review"
    }
    fn apply(&self, rating: &mut TestRating) {
        rating.review_text = Some("Great ride, very professional driver!".to_string());
    }
}

struct DriverRatingTrait;
impl FactoryTrait<TestRating> for DriverRatingTrait {
    fn name(&self) -> &str {
        "driver_rating"
    }
    fn apply(&self, rating: &mut TestRating) {
        rating.role_rated = "driver".to_string();
    }
}

struct PassengerRatingTrait;
impl FactoryTrait<TestRating> for PassengerRatingTrait {
    fn name(&self) -> &str {
        "passenger_rating"
    }
    fn apply(&self, rating: &mut TestRating) {
        rating.role_rated = "passenger".to_string();
    }
}
