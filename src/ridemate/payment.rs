use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::builder::BuildableFactory;
use crate::context::FactoryContext;
use crate::traits::{FactoryTrait, TraitRegistry};
use crate::Result;

/// Payment method entity matching the backend `payment_methods` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_payment_method_id: String,
    pub stripe_customer_id: String,
    pub card_brand: String,
    pub card_last_four: String,
    pub card_exp_month: i32,
    pub card_exp_year: i32,
    pub is_default: bool,
    pub nickname: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Wallet entity matching the backend `wallets` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance_cents: i64,
    pub pending_balance_cents: i64,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment entity matching the backend `payments` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPayment {
    pub id: Uuid,
    pub ride_id: Uuid,
    pub user_id: Uuid,
    pub payment_method_id: Option<Uuid>,
    pub stripe_payment_intent_id: Option<String>,
    pub amount_cents: i64,
    pub platform_fee_cents: i64,
    pub driver_payout_cents: i64,
    pub tip_cents: i64,
    pub currency: String,
    pub status: String,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// --- PaymentMethodFactory ---

pub struct PaymentMethodFactory {
    traits: TraitRegistry<TestPaymentMethod>,
}

impl PaymentMethodFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(VisaCardTrait));
        traits.register(Box::new(MastercardTrait));
        Self { traits }
    }
}

impl Default for PaymentMethodFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestPaymentMethod> for PaymentMethodFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestPaymentMethod {
        let n = ctx.sequence("payment_method");
        let now = Utc::now();

        TestPaymentMethod {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            stripe_payment_method_id: format!("pm_test_{}", Uuid::new_v4()),
            stripe_customer_id: format!("cus_test_{}", Uuid::new_v4()),
            card_brand: "visa".to_string(),
            card_last_four: format!("{:04}", 4242 + n),
            card_exp_month: 12,
            card_exp_year: 2028,
            is_default: true,
            nickname: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestPaymentMethod> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestPaymentMethod,
        overrides: &[(String, serde_json::Value)],
    ) {
        for (field, value) in overrides {
            match field.as_str() {
                "user_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.user_id = uuid;
                        }
                    }
                }
                "card_brand" => {
                    if let Some(v) = value.as_str() {
                        entity.card_brand = v.to_string();
                    }
                }
                "is_default" => {
                    if let Some(v) = value.as_bool() {
                        entity.is_default = v;
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(
        &self,
        entity: TestPaymentMethod,
        ctx: &mut FactoryContext,
    ) -> Result<TestPaymentMethod> {
        if ctx.http_client.is_some() {
            let body = serde_json::json!({
                "user_id": entity.user_id.to_string(),
                "card_brand": entity.card_brand,
                "card_last_four": entity.card_last_four,
                "is_default": entity.is_default,
            });
            ctx.test_post("/__test__/payments", &body).await?;
            return Ok(entity);
        }

        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            sqlx::query(
                r#"
                INSERT INTO payment_methods (
                    id, user_id, stripe_payment_method_id, stripe_customer_id,
                    card_brand, card_last_four, card_exp_month, card_exp_year,
                    is_default, nickname, is_active, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#,
            )
            .bind(entity.id)
            .bind(entity.user_id)
            .bind(&entity.stripe_payment_method_id)
            .bind(&entity.stripe_customer_id)
            .bind(&entity.card_brand)
            .bind(&entity.card_last_four)
            .bind(entity.card_exp_month)
            .bind(entity.card_exp_year)
            .bind(entity.is_default)
            .bind(&entity.nickname)
            .bind(entity.is_active)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- WalletFactory ---

pub struct WalletFactory {
    traits: TraitRegistry<TestWallet>,
}

impl WalletFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(FundedWalletTrait));
        traits.register(Box::new(EmptyWalletTrait));
        Self { traits }
    }
}

impl Default for WalletFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestWallet> for WalletFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestWallet {
        let _n = ctx.sequence("wallet");
        let now = Utc::now();

        TestWallet {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            balance_cents: 5000,
            pending_balance_cents: 0,
            is_active: true,
            is_verified: false,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestWallet> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestWallet,
        overrides: &[(String, serde_json::Value)],
    ) {
        for (field, value) in overrides {
            match field.as_str() {
                "user_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.user_id = uuid;
                        }
                    }
                }
                "balance_cents" => {
                    if let Some(v) = value.as_i64() {
                        entity.balance_cents = v;
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(&self, entity: TestWallet, ctx: &mut FactoryContext) -> Result<TestWallet> {
        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            sqlx::query(
                r#"
                INSERT INTO wallets (id, user_id, balance_cents, pending_balance_cents, is_active, is_verified, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (user_id) DO UPDATE SET balance_cents = $3, updated_at = $8
                "#,
            )
            .bind(entity.id)
            .bind(entity.user_id)
            .bind(entity.balance_cents)
            .bind(entity.pending_balance_cents)
            .bind(entity.is_active)
            .bind(entity.is_verified)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- PaymentFactory ---

pub struct PaymentFactory {
    traits: TraitRegistry<TestPayment>,
}

impl PaymentFactory {
    pub fn new() -> Self {
        let mut traits = TraitRegistry::new();
        traits.register(Box::new(SuccessfulPaymentTrait));
        traits.register(Box::new(FailedPaymentTrait));
        traits.register(Box::new(RefundedPaymentTrait));
        traits.register(Box::new(PendingPaymentTrait));
        Self { traits }
    }
}

impl Default for PaymentFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildableFactory<TestPayment> for PaymentFactory {
    fn build_base(&self, ctx: &mut FactoryContext) -> TestPayment {
        let _n = ctx.sequence("payment");
        let now = Utc::now();

        TestPayment {
            id: Uuid::new_v4(),
            ride_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            payment_method_id: None,
            stripe_payment_intent_id: Some(format!("pi_test_{}", Uuid::new_v4())),
            amount_cents: 2500,
            platform_fee_cents: 500,
            driver_payout_cents: 2000,
            tip_cents: 0,
            currency: "USD".to_string(),
            status: "succeeded".to_string(),
            failure_code: None,
            failure_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn trait_registry(&self) -> &TraitRegistry<TestPayment> {
        &self.traits
    }

    fn apply_overrides(
        &self,
        entity: &mut TestPayment,
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
                "user_id" => {
                    if let Some(v) = value.as_str() {
                        if let Ok(uuid) = Uuid::parse_str(v) {
                            entity.user_id = uuid;
                        }
                    }
                }
                "amount_cents" => {
                    if let Some(v) = value.as_i64() {
                        entity.amount_cents = v;
                    }
                }
                "status" => {
                    if let Some(v) = value.as_str() {
                        entity.status = v.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    async fn persist(&self, entity: TestPayment, ctx: &mut FactoryContext) -> Result<TestPayment> {
        #[cfg(feature = "postgres")]
        if let Some(pool) = &ctx.pool {
            sqlx::query(
                r#"
                INSERT INTO payments (
                    id, ride_id, user_id, payment_method_id, stripe_payment_intent_id,
                    amount_cents, platform_fee_cents, driver_payout_cents, tip_cents,
                    currency, status, failure_code, failure_message,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                "#,
            )
            .bind(entity.id)
            .bind(entity.ride_id)
            .bind(entity.user_id)
            .bind(entity.payment_method_id)
            .bind(&entity.stripe_payment_intent_id)
            .bind(entity.amount_cents)
            .bind(entity.platform_fee_cents)
            .bind(entity.driver_payout_cents)
            .bind(entity.tip_cents)
            .bind(&entity.currency)
            .bind(&entity.status)
            .bind(&entity.failure_code)
            .bind(&entity.failure_message)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .execute(pool)
            .await?;
            return Ok(entity);
        }

        Ok(entity)
    }
}

// --- Payment Traits ---

struct VisaCardTrait;
impl FactoryTrait<TestPaymentMethod> for VisaCardTrait {
    fn name(&self) -> &str {
        "visa"
    }
    fn apply(&self, pm: &mut TestPaymentMethod) {
        pm.card_brand = "visa".to_string();
        pm.card_last_four = "4242".to_string();
    }
}

struct MastercardTrait;
impl FactoryTrait<TestPaymentMethod> for MastercardTrait {
    fn name(&self) -> &str {
        "mastercard"
    }
    fn apply(&self, pm: &mut TestPaymentMethod) {
        pm.card_brand = "mastercard".to_string();
        pm.card_last_four = "5555".to_string();
    }
}

struct FundedWalletTrait;
impl FactoryTrait<TestWallet> for FundedWalletTrait {
    fn name(&self) -> &str {
        "funded"
    }
    fn apply(&self, wallet: &mut TestWallet) {
        wallet.balance_cents = 10000;
        wallet.is_verified = true;
    }
}

struct EmptyWalletTrait;
impl FactoryTrait<TestWallet> for EmptyWalletTrait {
    fn name(&self) -> &str {
        "empty"
    }
    fn apply(&self, wallet: &mut TestWallet) {
        wallet.balance_cents = 0;
    }
}

struct SuccessfulPaymentTrait;
impl FactoryTrait<TestPayment> for SuccessfulPaymentTrait {
    fn name(&self) -> &str {
        "successful"
    }
    fn apply(&self, payment: &mut TestPayment) {
        payment.status = "succeeded".to_string();
    }
}

struct FailedPaymentTrait;
impl FactoryTrait<TestPayment> for FailedPaymentTrait {
    fn name(&self) -> &str {
        "failed"
    }
    fn apply(&self, payment: &mut TestPayment) {
        payment.status = "failed".to_string();
        payment.failure_code = Some("card_declined".to_string());
        payment.failure_message = Some("Your card was declined".to_string());
    }
}

struct RefundedPaymentTrait;
impl FactoryTrait<TestPayment> for RefundedPaymentTrait {
    fn name(&self) -> &str {
        "refunded"
    }
    fn apply(&self, payment: &mut TestPayment) {
        payment.status = "refunded".to_string();
    }
}

struct PendingPaymentTrait;
impl FactoryTrait<TestPayment> for PendingPaymentTrait {
    fn name(&self) -> &str {
        "pending"
    }
    fn apply(&self, payment: &mut TestPayment) {
        payment.status = "pending".to_string();
    }
}
