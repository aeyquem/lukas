//! Persisted daily foreign-exchange observations for foreign assets.
//!
//! Debt opening values are deliberately handled elsewhere: a foreign opening
//! liability uses its separately confirmed CLP basis and must never be valued
//! through this daily asset-rate service.

use std::{future::Future, pin::Pin, str::FromStr, time::Duration};

use chrono::NaiveDate;
use num_bigint::BigUint;
use rust_decimal::Decimal;
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

use crate::financial_values::MoneyAmount;

pub const FRANKFURTER_SOURCE: &str = "Frankfurter v2";
const FRANKFURTER_URL: &str = "https://api.frankfurter.dev/v2/rate";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub type ProviderFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ProviderRate, ProviderError>> + Send + 'a>>;

/// A rate returned by a provider, before it is written to the workspace store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderRate {
    pub base_currency: String,
    pub quote_currency: String,
    pub effective_date: NaiveDate,
    pub rate: Decimal,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProviderError {
    #[error("FX provider request failed")]
    Unavailable,
    #[error("FX provider returned an invalid observation")]
    InvalidResponse,
}

/// A small injectable boundary keeps calculations and outage behavior
/// deterministic in tests while the production implementation uses HTTP.
pub trait RateProvider: Send + Sync {
    fn fetch<'a>(
        &'a self,
        base_currency: &'a str,
        quote_currency: &'a str,
        requested_date: NaiveDate,
    ) -> ProviderFuture<'a>;
}

#[derive(Clone)]
pub struct FrankfurterProvider {
    client: reqwest::Client,
}

impl FrankfurterProvider {
    pub fn new() -> Result<Self, FxError> {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|_| FxError::ProviderUnavailable)?;
        Ok(Self { client })
    }
}

impl RateProvider for FrankfurterProvider {
    fn fetch<'a>(
        &'a self,
        base_currency: &'a str,
        quote_currency: &'a str,
        requested_date: NaiveDate,
    ) -> ProviderFuture<'a> {
        Box::pin(async move {
            let url = format!("{FRANKFURTER_URL}/{base_currency}/{quote_currency}");
            let response = self
                .client
                .get(url)
                .query(&[("date", requested_date.format("%Y-%m-%d").to_string())])
                .send()
                .await
                .map_err(|_| ProviderError::Unavailable)?
                .error_for_status()
                .map_err(|_| ProviderError::Unavailable)?;
            let body = response
                .text()
                .await
                .map_err(|_| ProviderError::Unavailable)?;
            parse_provider_response(&body)
        })
    }
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct FxObservation {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub source: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub effective_date: NaiveDate,
    pub rate: Decimal,
    /// True when the provider could not supply a rate for the requested date,
    /// or its latest observation predates that date (for example, a weekend).
    pub stale: bool,
    /// True when the provider failed and a stored observation was used.
    /// Weekend and holiday observations can be stale while the provider works.
    pub provider_unavailable: bool,
}

#[derive(Debug, Error)]
pub enum FxError {
    #[error("foreign-exchange rate is unavailable")]
    RateUnavailable,
    #[error("foreign-exchange provider is unavailable")]
    ProviderUnavailable,
    #[error("foreign-exchange response is invalid")]
    InvalidProviderResponse,
    #[error("currency code is invalid")]
    InvalidCurrency,
    #[error("amount or rate is outside the supported decimal envelope")]
    AmountOutOfRange,
    #[error("foreign-asset valuation overflowed")]
    ArithmeticOverflow,
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),
}

impl From<sqlx::Error> for FxError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

/// Fetches the requested dated rate and persists its provider effective date.
/// On provider failure, the newest stored observation at or before the
/// requested date is returned as stale. If neither exists, the result is
/// unavailable instead of a zero rate.
pub async fn get_observation<P: RateProvider>(
    pool: &PgPool,
    workspace_id: Uuid,
    base_currency: &str,
    quote_currency: &str,
    requested_date: NaiveDate,
    provider: &P,
) -> Result<FxObservation, FxError> {
    validate_currency(base_currency)?;
    validate_currency(quote_currency)?;
    if base_currency == quote_currency || quote_currency != "CLP" {
        return Err(FxError::InvalidCurrency);
    }

    if let Ok(observation) = provider
        .fetch(base_currency, quote_currency, requested_date)
        .await
    {
        if validate_provider_rate(&observation, base_currency, quote_currency, requested_date)
            .is_ok()
        {
            return store_observation(pool, workspace_id, observation, requested_date).await;
        }
    }

    let mut fallback = latest_stored_observation(
        pool,
        workspace_id,
        base_currency,
        quote_currency,
        requested_date,
    )
    .await?
    .ok_or(FxError::ProviderUnavailable)?;
    fallback.provider_unavailable = true;
    Ok(fallback)
}

async fn store_observation(
    pool: &PgPool,
    workspace_id: Uuid,
    observation: ProviderRate,
    requested_date: NaiveDate,
) -> Result<FxObservation, FxError> {
    let id = Uuid::new_v4();
    let mut tx = pool.begin().await?;
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM workspaces WHERE id=$1 FOR UPDATE")
        .bind(workspace_id)
        .fetch_one(&mut *tx)
        .await?;
    let selected = sqlx::query_as::<_, StoredObservation>(
        "SELECT id, workspace_id, source, base_currency::text AS base_currency, \
                quote_currency::text AS quote_currency, effective_date, rate \
         FROM daily_fx_observations WHERE workspace_id=$1 AND source=$2 \
           AND base_currency=$3 AND quote_currency=$4 AND effective_date=$5 \
         ORDER BY observed_at DESC, id DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(FRANKFURTER_SOURCE)
    .bind(&observation.base_currency)
    .bind(&observation.quote_currency)
    .bind(observation.effective_date)
    .fetch_optional(&mut *tx)
    .await?;
    let existing = sqlx::query_as::<_, StoredObservation>(
        "SELECT id, workspace_id, source, base_currency::text AS base_currency, \
                quote_currency::text AS quote_currency, effective_date, rate \
         FROM daily_fx_observations WHERE workspace_id=$1 AND source=$2 \
           AND base_currency=$3 AND quote_currency=$4 AND effective_date=$5 AND rate=$6",
    )
    .bind(workspace_id)
    .bind(FRANKFURTER_SOURCE)
    .bind(&observation.base_currency)
    .bind(&observation.quote_currency)
    .bind(observation.effective_date)
    .bind(observation.rate)
    .fetch_optional(&mut *tx)
    .await?;
    let row = if let Some(existing) = existing {
        if selected.as_ref().is_some_and(|row| row.id != existing.id) {
            sqlx::query_as::<_, StoredObservation>(
                "UPDATE daily_fx_observations SET observed_at=clock_timestamp() \
                 WHERE workspace_id=$1 AND id=$2 \
                 RETURNING id, workspace_id, source, base_currency::text AS base_currency, \
                           quote_currency::text AS quote_currency, effective_date, rate",
            )
            .bind(workspace_id)
            .bind(existing.id)
            .fetch_one(&mut *tx)
            .await?
        } else {
            existing
        }
    } else {
        sqlx::query_as::<_, StoredObservation>(
            "INSERT INTO daily_fx_observations \
             (id, workspace_id, source, base_currency, quote_currency, effective_date, rate) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING id, workspace_id, source, base_currency::text AS base_currency, \
                       quote_currency::text AS quote_currency, effective_date, rate",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(FRANKFURTER_SOURCE)
        .bind(&observation.base_currency)
        .bind(&observation.quote_currency)
        .bind(observation.effective_date)
        .bind(observation.rate)
        .fetch_one(&mut *tx)
        .await?
    };
    tx.commit().await?;

    to_observation(row, observation.effective_date < requested_date)
}

async fn latest_stored_observation(
    pool: &PgPool,
    workspace_id: Uuid,
    base_currency: &str,
    quote_currency: &str,
    requested_date: NaiveDate,
) -> Result<Option<FxObservation>, FxError> {
    let row = sqlx::query_as::<_, StoredObservation>(
        "SELECT id, workspace_id, source, base_currency::text AS base_currency, \
                quote_currency::text AS quote_currency, effective_date, rate \
         FROM daily_fx_observations \
         WHERE workspace_id = $1 AND source = $2 AND base_currency = $3 \
           AND quote_currency = $4 AND effective_date <= $5 \
         ORDER BY effective_date DESC, observed_at DESC, id DESC LIMIT 1",
    )
    .bind(workspace_id)
    .bind(FRANKFURTER_SOURCE)
    .bind(base_currency)
    .bind(quote_currency)
    .bind(requested_date)
    .fetch_optional(pool)
    .await?;

    row.map(|stored| to_observation(stored, true)).transpose()
}

#[derive(FromRow)]
struct StoredObservation {
    id: Uuid,
    workspace_id: Uuid,
    source: String,
    base_currency: String,
    quote_currency: String,
    effective_date: NaiveDate,
    rate: Decimal,
}

fn to_observation(row: StoredObservation, stale: bool) -> Result<FxObservation, FxError> {
    checked_rate(row.rate)?;
    Ok(FxObservation {
        id: row.id,
        workspace_id: row.workspace_id,
        source: row.source,
        base_currency: row.base_currency.trim().to_owned(),
        quote_currency: row.quote_currency.trim().to_owned(),
        effective_date: row.effective_date,
        rate: row.rate,
        stale,
        provider_unavailable: false,
    })
}

/// Exact CLP value of a positive foreign asset amount at its dated rate.
/// Multiplication is exact; only the derived CLP value is rounded, once, to a
/// whole peso using midpoint-away-from-zero.
pub fn value_foreign_asset(
    amount: MoneyAmount,
    observation: &FxObservation,
) -> Result<MoneyAmount, FxError> {
    validate_currency(&observation.base_currency)?;
    if observation.quote_currency != "CLP" || amount.decimal() <= Decimal::ZERO {
        return Err(FxError::AmountOutOfRange);
    }
    checked_rate(observation.rate)?;
    // Decimal multiplication can rescale and round when the combined scale
    // exceeds 28 or the product needs more than 96 mantissa bits. Multiply
    // integer mantissas first so the only rounding happens at whole CLP.
    let amount_decimal = amount.decimal();
    let amount_mantissa =
        u128::try_from(amount_decimal.mantissa()).map_err(|_| FxError::AmountOutOfRange)?;
    let rate_mantissa =
        u128::try_from(observation.rate.mantissa()).map_err(|_| FxError::AmountOutOfRange)?;
    let product = BigUint::from(amount_mantissa) * BigUint::from(rate_mantissa);
    let scale = amount_decimal.scale() + observation.rate.scale();
    let divisor = BigUint::from(10u8).pow(scale);
    let quotient = &product / &divisor;
    let remainder = &product % &divisor;
    // Both operands are positive assets/rates, so midpoint-away-from-zero is
    // equivalent to incrementing at an exact half or above.
    let rounded = if remainder * BigUint::from(2u8) >= divisor {
        quotient + BigUint::from(1u8)
    } else {
        quotient
    };
    let rounded = rounded
        .to_str_radix(10)
        .parse::<i128>()
        .map_err(|_| FxError::AmountOutOfRange)?;
    let rounded =
        Decimal::try_from_i128_with_scale(rounded, 0).map_err(|_| FxError::AmountOutOfRange)?;
    MoneyAmount::new(rounded).map_err(|_| FxError::AmountOutOfRange)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForeignAssetInput {
    pub currency: String,
    pub amount: MoneyAmount,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetValuation {
    pub currency: String,
    pub amount: MoneyAmount,
    pub rate: Option<FxObservation>,
    pub clp_value: Option<MoneyAmount>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortfolioValuation {
    /// None means at least one eligible asset was unvalued. Known per-asset
    /// values remain explainable, but are never returned as a partial budget
    /// funding total.
    pub total_clp: Option<MoneyAmount>,
    pub missing_currencies: Vec<String>,
    pub assets: Vec<AssetValuation>,
}

/// Values foreign assets by currency. A missing required rate makes the
/// entire funding total unavailable while keeping each known rate/value
/// available for explanation.
pub fn value_foreign_assets(
    assets: &[ForeignAssetInput],
    observations: &[FxObservation],
) -> Result<PortfolioValuation, FxError> {
    let mut valuations = Vec::with_capacity(assets.len());
    let mut missing = std::collections::BTreeSet::new();
    let mut total = Decimal::ZERO;

    for asset in assets {
        validate_currency(&asset.currency)?;
        if asset.amount.decimal() <= Decimal::ZERO {
            return Err(FxError::AmountOutOfRange);
        }
        let observation = observations
            .iter()
            .find(|rate| rate.base_currency == asset.currency && rate.quote_currency == "CLP");
        match observation {
            Some(rate) => {
                let clp_value = value_foreign_asset(asset.amount, rate)?;
                total = total
                    .checked_add(clp_value.decimal())
                    .ok_or(FxError::ArithmeticOverflow)?;
                valuations.push(AssetValuation {
                    currency: asset.currency.clone(),
                    amount: asset.amount,
                    rate: Some(rate.clone()),
                    clp_value: Some(clp_value),
                });
            }
            None => {
                missing.insert(asset.currency.clone());
                valuations.push(AssetValuation {
                    currency: asset.currency.clone(),
                    amount: asset.amount,
                    rate: None,
                    clp_value: None,
                });
            }
        }
    }

    let total_clp = if missing.is_empty() {
        Some(MoneyAmount::new(total).map_err(|_| FxError::AmountOutOfRange)?)
    } else {
        None
    };
    Ok(PortfolioValuation {
        total_clp,
        missing_currencies: missing.into_iter().collect(),
        assets: valuations,
    })
}

fn validate_provider_rate(
    observation: &ProviderRate,
    requested_base: &str,
    requested_quote: &str,
    requested_date: NaiveDate,
) -> Result<(), FxError> {
    if observation.base_currency != requested_base
        || observation.quote_currency != requested_quote
        || observation.effective_date > requested_date
    {
        return Err(FxError::InvalidProviderResponse);
    }
    checked_rate(observation.rate)
}

fn checked_rate(rate: Decimal) -> Result<(), FxError> {
    if rate <= Decimal::ZERO {
        return Err(FxError::InvalidProviderResponse);
    }
    MoneyAmount::new(rate).map_err(|_| FxError::AmountOutOfRange)?;
    Ok(())
}

fn validate_currency(currency: &str) -> Result<(), FxError> {
    if currency.len() != 3 || !currency.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Err(FxError::InvalidCurrency);
    }
    Ok(())
}

fn parse_provider_response(body: &str) -> Result<ProviderRate, ProviderError> {
    let value: Value = serde_json::from_str(body).map_err(|_| ProviderError::InvalidResponse)?;
    let date_text = value
        .get("date")
        .and_then(Value::as_str)
        .ok_or(ProviderError::InvalidResponse)?;
    let effective_date = NaiveDate::parse_from_str(date_text, "%Y-%m-%d")
        .map_err(|_| ProviderError::InvalidResponse)?;
    let base_currency = value
        .get("base")
        .and_then(Value::as_str)
        .ok_or(ProviderError::InvalidResponse)?;
    let quote_currency = value
        .get("quote")
        .and_then(Value::as_str)
        .ok_or(ProviderError::InvalidResponse)?;
    let rate_text = value
        .get("rate")
        .and_then(Value::as_number)
        .map(ToString::to_string)
        .ok_or(ProviderError::InvalidResponse)?;
    let rate = Decimal::from_str(&rate_text).map_err(|_| ProviderError::InvalidResponse)?;

    Ok(ProviderRate {
        base_currency: base_currency.to_owned(),
        quote_currency: quote_currency.to_owned(),
        effective_date,
        rate,
    })
}

#[cfg(test)]
mod tests {
    use std::{future::Future, pin::Pin};

    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{financial_values::MoneyAmount, workspace::bootstrap};

    use super::{
        FRANKFURTER_SOURCE, ForeignAssetInput, FxError, FxObservation, ProviderError, ProviderRate,
        RateProvider, get_observation, parse_provider_response, value_foreign_asset,
        value_foreign_assets,
    };

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn decimal(value: &str) -> Decimal {
        Decimal::from_str_exact(value).unwrap()
    }

    fn money(value: &str) -> MoneyAmount {
        MoneyAmount::parse(value).unwrap()
    }

    fn observation(currency: &str, rate: &str, stale: bool) -> FxObservation {
        FxObservation {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            source: FRANKFURTER_SOURCE.to_owned(),
            base_currency: currency.to_owned(),
            quote_currency: "CLP".to_owned(),
            effective_date: date(2026, 11, 5),
            rate: decimal(rate),
            stale,
            provider_unavailable: false,
        }
    }

    struct StaticProvider(Result<ProviderRate, ProviderError>);

    impl RateProvider for StaticProvider {
        fn fetch<'a>(
            &'a self,
            _base_currency: &'a str,
            _quote_currency: &'a str,
            _requested_date: NaiveDate,
        ) -> Pin<Box<dyn Future<Output = Result<ProviderRate, ProviderError>> + Send + 'a>>
        {
            Box::pin(async move { self.0.clone() })
        }
    }

    #[test]
    fn frankfurter_response_keeps_decimal_rate_and_actual_effective_date() {
        let parsed = parse_provider_response(
            r#"{"date":"2026-11-05","base":"USD","quote":"CLP","rate":935.125}"#,
        )
        .unwrap();
        assert_eq!(parsed.base_currency, "USD");
        assert_eq!(parsed.quote_currency, "CLP");
        assert_eq!(parsed.effective_date, date(2026, 11, 5));
        assert_eq!(parsed.rate, decimal("935.125"));
    }

    #[test]
    fn asset_values_round_once_to_whole_clp_pesos() {
        let rate = observation("USD", "800.5", false);
        assert_eq!(
            value_foreign_asset(money("1"), &rate).unwrap(),
            money("801")
        );
        assert_eq!(
            value_foreign_asset(money("1.5"), &observation("USD", "800.1", false)).unwrap(),
            money("1200")
        );
    }

    #[test]
    fn negative_assets_and_liability_rates_are_rejected() {
        assert!(matches!(
            value_foreign_asset(money("-1"), &observation("USD", "900", false)),
            Err(FxError::AmountOutOfRange)
        ));
        let mut debt_rate = observation("USD", "900", false);
        debt_rate.quote_currency = "EUR".to_owned();
        assert!(matches!(
            value_foreign_asset(money("1"), &debt_rate),
            Err(FxError::AmountOutOfRange)
        ));
    }

    #[test]
    fn missing_asset_rate_makes_the_funding_total_unavailable() {
        let assets = vec![
            ForeignAssetInput {
                currency: "USD".to_owned(),
                amount: money("100"),
            },
            ForeignAssetInput {
                currency: "EUR".to_owned(),
                amount: money("50"),
            },
        ];
        let result = value_foreign_assets(&assets, &[observation("USD", "900", false)]).unwrap();
        assert_eq!(result.total_clp, None);
        assert_eq!(result.missing_currencies, vec!["EUR"]);
        assert_eq!(result.assets[0].clp_value, Some(money("90000")));
        assert_eq!(result.assets[1].clp_value, None);
    }

    #[test]
    fn all_valued_assets_produce_a_checked_total() {
        let assets = vec![
            ForeignAssetInput {
                currency: "USD".to_owned(),
                amount: money("10"),
            },
            ForeignAssetInput {
                currency: "EUR".to_owned(),
                amount: money("20"),
            },
        ];
        let result = value_foreign_assets(
            &assets,
            &[
                observation("USD", "900", true),
                observation("EUR", "1000", false),
            ],
        )
        .unwrap();
        assert_eq!(result.total_clp, Some(money("29000")));
        assert!(result.missing_currencies.is_empty());
        assert!(result.assets[0].rate.as_ref().unwrap().stale);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn persists_provider_effective_date_and_uses_latest_stored_rate_when_unavailable(
        pool: PgPool,
    ) {
        let workspace_id = bootstrap(&pool).await.unwrap();
        let fetched_date = date(2026, 11, 5);
        let requested_date = date(2026, 11, 7);
        let provider = StaticProvider(Ok(ProviderRate {
            base_currency: "USD".to_owned(),
            quote_currency: "CLP".to_owned(),
            effective_date: fetched_date,
            rate: decimal("935.125"),
        }));
        let fetched = get_observation(&pool, workspace_id, "USD", "CLP", fetched_date, &provider)
            .await
            .unwrap();
        assert_eq!(fetched.effective_date, fetched_date);
        assert!(!fetched.stale);

        let same_rate = get_observation(
            &pool,
            workspace_id,
            "USD",
            "CLP",
            fetched_date,
            &StaticProvider(Ok(ProviderRate {
                base_currency: "USD".to_owned(),
                quote_currency: "CLP".to_owned(),
                effective_date: fetched_date,
                rate: decimal("935.125"),
            })),
        )
        .await
        .unwrap();
        assert_eq!(same_rate.id, fetched.id);

        let account_id = Uuid::new_v4();
        let wallet_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO financial_accounts \
             (id, workspace_id, name, account_type, base_currency, budget_eligible) \
             VALUES ($1, $2, 'FX test', 'bank', 'CLP', TRUE)",
        )
        .bind(account_id)
        .bind(workspace_id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO financial_wallets (id, workspace_id, account_id, currency) \
             VALUES ($1, $2, $3, 'USD')",
        )
        .bind(wallet_id)
        .bind(workspace_id)
        .bind(account_id)
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO budget_funding_effects \
             (id, workspace_id, period_start, source_kind, wallet_id, amount, currency, rate_observation_id) \
             VALUES ($1, $2, '2026-11-01', 'valuation', $3, '100', 'USD', $4)",
        )
        .bind(Uuid::new_v4())
        .bind(workspace_id)
        .bind(wallet_id)
        .bind(fetched.id)
        .execute(&pool)
        .await
        .unwrap();

        let revised_rate = get_observation(
            &pool,
            workspace_id,
            "USD",
            "CLP",
            fetched_date,
            &StaticProvider(Ok(ProviderRate {
                base_currency: "USD".to_owned(),
                quote_currency: "CLP".to_owned(),
                effective_date: fetched_date,
                rate: decimal("936.125"),
            })),
        )
        .await
        .unwrap();
        assert_ne!(revised_rate.id, fetched.id);
        assert_eq!(revised_rate.rate, decimal("936.125"));
        assert!(!revised_rate.provider_unavailable);

        let repeated_revision = get_observation(
            &pool,
            workspace_id,
            "USD",
            "CLP",
            fetched_date,
            &StaticProvider(Ok(ProviderRate {
                base_currency: "USD".to_owned(),
                quote_currency: "CLP".to_owned(),
                effective_date: fetched_date,
                rate: decimal("936.125"),
            })),
        )
        .await
        .unwrap();
        assert_eq!(repeated_revision.id, revised_rate.id);

        let reverted_rate = get_observation(
            &pool,
            workspace_id,
            "USD",
            "CLP",
            fetched_date,
            &StaticProvider(Ok(ProviderRate {
                base_currency: "USD".to_owned(),
                quote_currency: "CLP".to_owned(),
                effective_date: fetched_date,
                rate: decimal("935.125"),
            })),
        )
        .await
        .unwrap();
        assert_eq!(reverted_rate.id, fetched.id);

        let original_rate: Decimal = sqlx::query_scalar(
            "SELECT rate FROM daily_fx_observations WHERE workspace_id = $1 AND id = $2",
        )
        .bind(workspace_id)
        .bind(fetched.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(original_rate, decimal("935.125"));
        let effect_rate_id: Uuid = sqlx::query_scalar(
            "SELECT rate_observation_id FROM budget_funding_effects \
             WHERE workspace_id = $1 AND rate_observation_id = $2",
        )
        .bind(workspace_id)
        .bind(fetched.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(effect_rate_id, fetched.id);

        let stored_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM daily_fx_observations \
             WHERE workspace_id = $1 AND source = $2 AND base_currency = 'USD' \
               AND quote_currency = 'CLP' AND effective_date = $3",
        )
        .bind(workspace_id)
        .bind(FRANKFURTER_SOURCE)
        .bind(fetched_date)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stored_count, 2);

        let unavailable = StaticProvider(Err(ProviderError::Unavailable));
        let fallback = get_observation(
            &pool,
            workspace_id,
            "USD",
            "CLP",
            requested_date,
            &unavailable,
        )
        .await
        .unwrap();
        assert_eq!(fallback.id, fetched.id);
        assert_eq!(fallback.rate, decimal("935.125"));
        assert_eq!(fallback.effective_date, fetched_date);
        assert!(fallback.stale);
        assert!(fallback.provider_unavailable);

        let no_prior_observation = get_observation(
            &pool,
            workspace_id,
            "EUR",
            "CLP",
            requested_date,
            &unavailable,
        )
        .await;
        assert!(matches!(
            no_prior_observation,
            Err(FxError::ProviderUnavailable)
        ));
    }
}
