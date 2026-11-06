//! Pure, checked calculations for first-slice monthly budgets.
//!
//! The functions in this module accept dated financial facts and saved
//! assignments. They do not read account balances, persist results, or infer a
//! value when a required conversion is missing.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::{Decimal, RoundingStrategy};

use crate::financial_values::MoneyAmount;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalculationError {
    InvalidPeriod,
    AmountOutOfRange,
    ArithmeticOverflow,
    NegativeAssignment,
}

pub type CategoryId = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BudgetPeriod {
    year: i32,
    month: u32,
}

impl BudgetPeriod {
    pub fn new(year: i32, month: u32) -> Result<Self, CalculationError> {
        if NaiveDate::from_ymd_opt(year, month, 1).is_none() {
            return Err(CalculationError::InvalidPeriod);
        }
        Ok(Self { year, month })
    }

    pub fn from_date(date: NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }

    pub fn contains(self, date: NaiveDate) -> bool {
        date.year() == self.year && date.month() == self.month
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingValue {
    ForeignAssetRate,
    ForeignLiabilityBasis,
    TransferSourceValuation,
    TransferDestinationValuation,
    TransactionClpValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueState {
    Known(Decimal),
    Missing(MissingValue),
}

impl ValueState {
    pub fn known(value: Decimal) -> Result<Self, CalculationError> {
        checked_amount(value)?;
        Ok(Self::Known(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FundingKind {
    Opening,
    OpeningCorrection,
    Income,
    Transfer,
    ForeignAssetRevaluation,
    OtherCorrection,
}

/// A distinct, already-denominated funding event. Corrections carry a net
/// signed delta, never the full corrected opening or valuation amount.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FundingEvent {
    pub date: NaiveDate,
    pub kind: FundingKind,
    pub eligible: bool,
    pub value_clp: ValueState,
}

/// For an eligible foreign asset, daily valuation is rounded once to whole
/// pesos using midpoint-away-from-zero. `amount` retains the original
/// currency value; `clp_per_unit` is the dated applicable observation.
pub fn foreign_asset_opening_value(
    amount: Decimal,
    clp_per_unit: Option<Decimal>,
) -> Result<ValueState, CalculationError> {
    checked_amount(amount)?;
    if amount < Decimal::ZERO {
        return Err(CalculationError::AmountOutOfRange);
    }
    let Some(rate) = clp_per_unit else {
        return Ok(ValueState::Missing(MissingValue::ForeignAssetRate));
    };
    if rate <= Decimal::ZERO {
        return Err(CalculationError::AmountOutOfRange);
    }
    let raw = amount
        .checked_mul(rate)
        .ok_or(CalculationError::ArithmeticOverflow)?;
    let pesos = raw.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero);
    checked_amount(pesos)?;
    Ok(ValueState::Known(pesos))
}

/// An opening liability uses its separately supplied signed CLP basis. Daily
/// debt FX is intentionally not accepted by this function.
pub fn foreign_liability_opening_value(
    signed_clp_basis: Option<Decimal>,
) -> Result<ValueState, CalculationError> {
    match signed_clp_basis {
        Some(value) => {
            checked_amount(value)?;
            Ok(ValueState::Known(value))
        }
        None => Ok(ValueState::Missing(MissingValue::ForeignLiabilityBasis)),
    }
}

/// Returns the signed difference between a corrected opening's new and
/// previously applied funding value. Each opening edit should persist this
/// result as one correction event.
pub fn opening_correction_delta(
    previously_applied_clp: Decimal,
    corrected_clp: Decimal,
) -> Result<Decimal, CalculationError> {
    checked_amount(previously_applied_clp)?;
    checked_amount(corrected_clp)?;
    checked_sub(corrected_clp, previously_applied_clp)
}

/// Returns the single budget-boundary effect of a logical transfer. Source
/// values are the prior CLP carrying value; destination values are actual
/// received CLP value or the accepted transaction-date valuation. A
/// tracking-only side does not need a valuation because it has no budget
/// effect. An incomplete eligible side makes the whole transfer incomplete.
pub fn transfer_funding_effect(
    source_eligible: bool,
    source_value_clp: ValueState,
    destination_eligible: bool,
    destination_value_clp: ValueState,
) -> Result<ValueState, CalculationError> {
    let source = if source_eligible {
        match source_value_clp {
            ValueState::Known(value) => {
                validate_transfer_value(value)?;
                Some(checked_neg(value)?)
            }
            ValueState::Missing(_) => {
                return Ok(ValueState::Missing(MissingValue::TransferSourceValuation));
            }
        }
    } else {
        None
    };
    let destination = if destination_eligible {
        match destination_value_clp {
            ValueState::Known(value) => {
                validate_transfer_value(value)?;
                Some(value)
            }
            ValueState::Missing(_) => {
                return Ok(ValueState::Missing(
                    MissingValue::TransferDestinationValuation,
                ));
            }
        }
    } else {
        None
    };
    match (source, destination) {
        (None, None) => Ok(ValueState::Known(Decimal::ZERO)),
        (Some(value), None) | (None, Some(value)) => {
            checked_amount(value)?;
            Ok(ValueState::Known(value))
        }
        (Some(outflow), Some(inflow)) => Ok(ValueState::Known(checked_add(outflow, inflow)?)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivityKind {
    Income,
    Expense,
    Refund,
    Transfer,
}

/// Activity is entered once per logical transaction. Refunds linked to an
/// expense should use that expense's category; an unlinked refund may use its
/// explicitly selected category. Transfers have no category activity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionActivity {
    pub date: NaiveDate,
    pub kind: ActivityKind,
    pub eligible: bool,
    pub category_id: Option<CategoryId>,
    /// Positive CLP magnitude; `None` represents a foreign expense/refund
    /// whose statement value has not yet been confirmed.
    pub amount_clp: Option<Decimal>,
    /// Original currency and amount remain available for explaining an
    /// incomplete CLP activity value.
    pub original_currency: Option<String>,
    pub original_amount: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedActivity {
    pub kind: ActivityKind,
    pub category_id: Option<CategoryId>,
    pub currency: Option<String>,
    pub amount: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivitySummary {
    /// Positive means spending; a refund reduces spending.
    pub category_activity: BTreeMap<CategoryId, Decimal>,
    pub uncategorized_activity_clp: Decimal,
    pub unresolved: Vec<UnresolvedActivity>,
    pub funding_events: Vec<FundingEvent>,
}

pub fn attribute_transactions(
    transactions: &[TransactionActivity],
) -> Result<ActivitySummary, CalculationError> {
    let mut summary = ActivitySummary {
        category_activity: BTreeMap::new(),
        uncategorized_activity_clp: Decimal::ZERO,
        unresolved: Vec::new(),
        funding_events: Vec::new(),
    };

    for transaction in transactions {
        if transaction.kind == ActivityKind::Transfer {
            continue;
        }
        let Some(amount) = transaction.amount_clp else {
            if transaction.kind == ActivityKind::Income && transaction.eligible {
                summary.funding_events.push(FundingEvent {
                    date: transaction.date,
                    kind: FundingKind::Income,
                    eligible: true,
                    value_clp: ValueState::Missing(MissingValue::TransactionClpValue),
                });
            } else if matches!(
                transaction.kind,
                ActivityKind::Expense | ActivityKind::Refund
            ) {
                summary.unresolved.push(UnresolvedActivity {
                    kind: transaction.kind.clone(),
                    category_id: transaction.category_id.clone(),
                    currency: transaction.original_currency.clone(),
                    amount: transaction.original_amount,
                });
            }
            continue;
        };
        if amount <= Decimal::ZERO {
            return Err(CalculationError::AmountOutOfRange);
        }
        checked_amount(amount)?;

        match transaction.kind {
            ActivityKind::Income => {
                if transaction.eligible {
                    summary.funding_events.push(FundingEvent {
                        date: transaction.date,
                        kind: FundingKind::Income,
                        eligible: true,
                        value_clp: ValueState::Known(amount),
                    });
                }
            }
            ActivityKind::Expense | ActivityKind::Refund => {
                let signed = if transaction.kind == ActivityKind::Expense {
                    amount
                } else {
                    checked_neg(amount)?
                };
                if let Some(category_id) = &transaction.category_id {
                    let current = summary
                        .category_activity
                        .get(category_id)
                        .copied()
                        .unwrap_or(Decimal::ZERO);
                    summary
                        .category_activity
                        .insert(category_id.clone(), checked_add(current, signed)?);
                } else {
                    summary.uncategorized_activity_clp =
                        checked_add(summary.uncategorized_activity_clp, signed)?;
                }
            }
            ActivityKind::Transfer => unreachable!("transfers returned above"),
        }
    }

    Ok(summary)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalculatedValue {
    Complete(Decimal),
    /// Required inputs are missing; a partial funding total is not exposed.
    Unavailable(Vec<MissingValue>),
    Incomplete {
        known_subtotal: Decimal,
        unresolved: Vec<UnresolvedActivity>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodInput {
    pub period: BudgetPeriod,
    pub prior_unassigned_clp: Decimal,
    /// Signed remainders carried in their categories. Categories released at
    /// close belong in `released_category_remainders_clp` instead.
    pub prior_category_remaining_clp: BTreeMap<CategoryId, Decimal>,
    /// Signed remainders released by the immediately preceding close.
    pub released_category_remainders_clp: BTreeMap<CategoryId, Decimal>,
    /// Dated openings, linked net corrections, transfer-boundary effects, and
    /// valuation deltas. Recognized income is supplied through `activities`
    /// and should not be repeated here.
    pub funding_events: Vec<FundingEvent>,
    pub activities: Vec<TransactionActivity>,
    /// Assignments are saved intent and may exceed available funding.
    pub assignments_clp: BTreeMap<CategoryId, Decimal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodResult {
    pub period: BudgetPeriod,
    pub ready_to_assign_clp: CalculatedValue,
    pub category_activity_clp: BTreeMap<CategoryId, CalculatedValue>,
    pub uncategorized_activity_clp: CalculatedValue,
    pub category_remaining_clp: BTreeMap<CategoryId, CalculatedValue>,
    pub unresolved_activity: Vec<UnresolvedActivity>,
    pub can_close: bool,
}

pub fn calculate_period(input: &PeriodInput) -> Result<PeriodResult, CalculationError> {
    checked_amount(input.prior_unassigned_clp)?;
    let current_activities = input
        .activities
        .iter()
        .filter(|transaction| input.period.contains(transaction.date))
        .cloned()
        .collect::<Vec<_>>();
    let activity = attribute_transactions(&current_activities)?;

    let mut known_funding = Decimal::ZERO;
    let mut missing_funding = Vec::new();
    for event in input
        .funding_events
        .iter()
        .chain(activity.funding_events.iter())
    {
        if !input.period.contains(event.date) || !event.eligible {
            continue;
        }
        match event.value_clp {
            ValueState::Known(value) => {
                known_funding = checked_add(known_funding, value)?;
            }
            ValueState::Missing(reason) => missing_funding.push(reason),
        }
    }

    let mut unassigned = checked_add(input.prior_unassigned_clp, known_funding)?;
    for value in input.released_category_remainders_clp.values() {
        unassigned = checked_add(unassigned, *value)?;
    }
    for assignment in input.assignments_clp.values() {
        if *assignment < Decimal::ZERO {
            return Err(CalculationError::NegativeAssignment);
        }
        checked_amount(*assignment)?;
        unassigned = checked_sub(unassigned, *assignment)?;
    }
    let ready_to_assign_clp = if missing_funding.is_empty() {
        CalculatedValue::Complete(unassigned)
    } else {
        CalculatedValue::Unavailable(missing_funding.clone())
    };

    let mut category_ids = BTreeMap::<CategoryId, ()>::new();
    for id in input
        .prior_category_remaining_clp
        .keys()
        .chain(input.released_category_remainders_clp.keys())
        .chain(input.assignments_clp.keys())
        .chain(activity.category_activity.keys())
        .chain(
            activity
                .unresolved
                .iter()
                .filter_map(|entry| entry.category_id.as_ref()),
        )
    {
        category_ids.insert(id.clone(), ());
    }

    let mut category_activity_clp = BTreeMap::new();
    let mut category_remaining_clp = BTreeMap::new();
    let all_unresolved = activity.unresolved.clone();
    for id in category_ids.keys() {
        let current_activity = activity
            .category_activity
            .get(id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        let unresolved_for_category = activity
            .unresolved
            .iter()
            .filter(|entry| entry.category_id.as_ref() == Some(id))
            .cloned()
            .collect::<Vec<_>>();
        let activity_value = if unresolved_for_category.is_empty() {
            CalculatedValue::Complete(current_activity)
        } else {
            CalculatedValue::Incomplete {
                known_subtotal: current_activity,
                unresolved: unresolved_for_category.clone(),
            }
        };
        category_activity_clp.insert(id.clone(), activity_value);

        let previous = input
            .prior_category_remaining_clp
            .get(id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        let assignment = input
            .assignments_clp
            .get(id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        // Released remainders move to ready-to-assign at close. They no longer
        // belong to this category; only carried remainders stay in-category.
        let assigned_available = checked_add(previous, assignment)?;
        let remaining = checked_sub(assigned_available, current_activity)?;
        let unresolved_for_category = unresolved_for_category;
        let result = if unresolved_for_category.is_empty() {
            CalculatedValue::Complete(remaining)
        } else {
            CalculatedValue::Incomplete {
                known_subtotal: remaining,
                unresolved: unresolved_for_category,
            }
        };
        category_remaining_clp.insert(id.clone(), result);
    }

    let uncategorized_activity_clp = if activity
        .unresolved
        .iter()
        .any(|entry| entry.category_id.is_none())
    {
        CalculatedValue::Incomplete {
            known_subtotal: activity.uncategorized_activity_clp,
            unresolved: activity
                .unresolved
                .iter()
                .filter(|entry| entry.category_id.is_none())
                .cloned()
                .collect(),
        }
    } else {
        CalculatedValue::Complete(activity.uncategorized_activity_clp)
    };

    // Unresolved values in a period only affect that period's completeness.
    let can_close = all_unresolved.is_empty() && missing_funding.is_empty();

    Ok(PeriodResult {
        period: input.period,
        ready_to_assign_clp,
        category_activity_clp,
        uncategorized_activity_clp,
        category_remaining_clp,
        unresolved_activity: all_unresolved,
        can_close,
    })
}

fn checked_amount(value: Decimal) -> Result<(), CalculationError> {
    MoneyAmount::new(value)
        .map(|_| ())
        .map_err(|_| CalculationError::AmountOutOfRange)
}

fn validate_transfer_value(value: Decimal) -> Result<(), CalculationError> {
    checked_amount(value)?;
    if value < Decimal::ZERO {
        return Err(CalculationError::AmountOutOfRange);
    }
    Ok(())
}

fn checked_add(left: Decimal, right: Decimal) -> Result<Decimal, CalculationError> {
    let result = left
        .checked_add(right)
        .ok_or(CalculationError::ArithmeticOverflow)?;
    checked_amount(result)?;
    if result.checked_sub(left) != Some(right) || result.checked_sub(right) != Some(left) {
        return Err(CalculationError::ArithmeticOverflow);
    }
    Ok(result)
}

fn checked_sub(left: Decimal, right: Decimal) -> Result<Decimal, CalculationError> {
    let result = left
        .checked_sub(right)
        .ok_or(CalculationError::ArithmeticOverflow)?;
    checked_amount(result)?;
    if result.checked_add(right) != Some(left) || left.checked_sub(result) != Some(right) {
        return Err(CalculationError::ArithmeticOverflow);
    }
    Ok(result)
}

fn checked_neg(value: Decimal) -> Result<Decimal, CalculationError> {
    let result = Decimal::ZERO
        .checked_sub(value)
        .ok_or(CalculationError::ArithmeticOverflow)?;
    checked_amount(result)?;
    if result.checked_add(value) != Some(Decimal::ZERO) {
        return Err(CalculationError::ArithmeticOverflow);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    fn d(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    fn period_input() -> PeriodInput {
        PeriodInput {
            period: BudgetPeriod::new(2026, 1).unwrap(),
            prior_unassigned_clp: Decimal::ZERO,
            prior_category_remaining_clp: BTreeMap::new(),
            released_category_remainders_clp: BTreeMap::new(),
            funding_events: Vec::new(),
            activities: Vec::new(),
            assignments_clp: BTreeMap::new(),
        }
    }

    #[test]
    fn foreign_openings_preserve_signs_and_use_required_provenance() {
        assert_eq!(
            foreign_asset_opening_value(d("12.5"), Some(d("800.4"))).unwrap(),
            ValueState::Known(d("10005"))
        );
        assert_eq!(
            foreign_asset_opening_value(d("-12.5"), Some(d("800.4"))),
            Err(CalculationError::AmountOutOfRange)
        );
        assert_eq!(
            foreign_asset_opening_value(d("12.5"), None).unwrap(),
            ValueState::Missing(MissingValue::ForeignAssetRate)
        );
        assert_eq!(
            foreign_liability_opening_value(Some(d("-10000"))).unwrap(),
            ValueState::Known(d("-10000"))
        );
        assert_eq!(
            foreign_liability_opening_value(None).unwrap(),
            ValueState::Missing(MissingValue::ForeignLiabilityBasis)
        );
        assert_eq!(
            opening_correction_delta(d("10000"), d("8500")).unwrap(),
            d("-1500")
        );
    }

    #[test]
    fn transfer_boundary_counts_only_eligible_sides_once() {
        assert_eq!(
            transfer_funding_effect(
                true,
                ValueState::Known(d("100000")),
                true,
                ValueState::Known(d("95000")),
            )
            .unwrap(),
            ValueState::Known(d("-5000"))
        );
        assert_eq!(
            transfer_funding_effect(
                true,
                ValueState::Known(d("100000")),
                false,
                ValueState::Missing(MissingValue::TransferDestinationValuation),
            )
            .unwrap(),
            ValueState::Known(d("-100000"))
        );
        assert_eq!(
            transfer_funding_effect(
                false,
                ValueState::Missing(MissingValue::TransferSourceValuation),
                true,
                ValueState::Known(d("99000")),
            )
            .unwrap(),
            ValueState::Known(d("99000"))
        );
        assert_eq!(
            transfer_funding_effect(
                false,
                ValueState::Missing(MissingValue::TransferSourceValuation),
                false,
                ValueState::Missing(MissingValue::TransferDestinationValuation),
            )
            .unwrap(),
            ValueState::Known(Decimal::ZERO)
        );
        assert_eq!(
            transfer_funding_effect(
                true,
                ValueState::Known(d("-1")),
                false,
                ValueState::Known(Decimal::ZERO),
            ),
            Err(CalculationError::AmountOutOfRange)
        );
    }

    #[test]
    fn income_funds_once_but_expense_refund_and_transfer_do_not_fund() {
        let records = vec![
            TransactionActivity {
                date: date("2026-01-02"),
                kind: ActivityKind::Income,
                eligible: true,
                category_id: None,
                amount_clp: Some(d("1000")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("1000")),
            },
            TransactionActivity {
                date: date("2026-01-03"),
                kind: ActivityKind::Income,
                eligible: false,
                category_id: None,
                amount_clp: Some(d("500")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("500")),
            },
            TransactionActivity {
                date: date("2026-01-04"),
                kind: ActivityKind::Expense,
                eligible: true,
                category_id: Some("food".into()),
                amount_clp: Some(d("300")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("300")),
            },
            TransactionActivity {
                date: date("2026-01-05"),
                kind: ActivityKind::Refund,
                eligible: true,
                category_id: Some("food".into()),
                amount_clp: Some(d("50")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("50")),
            },
            TransactionActivity {
                date: date("2026-01-06"),
                kind: ActivityKind::Transfer,
                eligible: true,
                category_id: None,
                amount_clp: Some(d("900")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("900")),
            },
        ];
        let result = attribute_transactions(&records).unwrap();
        assert_eq!(result.funding_events.len(), 1);
        assert_eq!(
            result.funding_events[0].value_clp,
            ValueState::Known(d("1000"))
        );
        assert_eq!(result.category_activity["food"], d("250"));
    }

    #[test]
    fn period_keeps_negative_values_and_carries_signed_category_remainders() {
        let mut input = period_input();
        input.prior_unassigned_clp = d("100");
        input
            .prior_category_remaining_clp
            .insert("food".into(), d("-20"));
        input
            .released_category_remainders_clp
            .insert("rent".into(), d("30"));
        input.assignments_clp.insert("food".into(), d("150"));
        input.assignments_clp.insert("rent".into(), d("10"));
        input.assignments_clp.insert("transport".into(), d("5"));
        input.funding_events.push(FundingEvent {
            date: date("2026-01-10"),
            kind: FundingKind::Opening,
            eligible: true,
            value_clp: ValueState::Known(d("200")),
        });
        input.funding_events.push(FundingEvent {
            date: date("2026-01-10"),
            kind: FundingKind::Opening,
            eligible: true,
            value_clp: ValueState::Known(d("-20")),
        });
        input.activities.push(TransactionActivity {
            date: date("2026-01-11"),
            kind: ActivityKind::Expense,
            eligible: true,
            category_id: Some("food".into()),
            amount_clp: Some(d("40")),
            original_currency: Some("CLP".into()),
            original_amount: Some(d("40")),
        });
        input.activities.push(TransactionActivity {
            date: date("2026-01-12"),
            kind: ActivityKind::Expense,
            eligible: true,
            category_id: Some("transport".into()),
            amount_clp: Some(d("10")),
            original_currency: Some("CLP".into()),
            original_amount: Some(d("10")),
        });
        let result = calculate_period(&input).unwrap();
        assert_eq!(
            result.ready_to_assign_clp,
            CalculatedValue::Complete(d("145"))
        );
        assert_eq!(
            result.category_remaining_clp["food"],
            CalculatedValue::Complete(d("90"))
        );
        assert_eq!(
            result.category_remaining_clp["rent"],
            CalculatedValue::Complete(d("10"))
        );
        assert_eq!(
            result.category_remaining_clp["transport"],
            CalculatedValue::Complete(d("-5"))
        );
        assert!(result.can_close);
    }

    #[test]
    fn incomplete_foreign_expense_preserves_subtotal_and_withholds_close_and_funding_total() {
        let mut input = period_input();
        input.funding_events.push(FundingEvent {
            date: date("2026-01-01"),
            kind: FundingKind::Opening,
            eligible: true,
            value_clp: ValueState::Missing(MissingValue::ForeignAssetRate),
        });
        input.activities.extend([
            TransactionActivity {
                date: date("2026-01-01"),
                kind: ActivityKind::Expense,
                eligible: true,
                category_id: Some("food".into()),
                amount_clp: Some(d("100")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("100")),
            },
            TransactionActivity {
                date: date("2026-01-02"),
                kind: ActivityKind::Expense,
                eligible: true,
                category_id: Some("food".into()),
                amount_clp: None,
                original_currency: Some("USD".into()),
                original_amount: Some(d("2")),
            },
        ]);
        let result = calculate_period(&input).unwrap();
        assert_eq!(
            result.ready_to_assign_clp,
            CalculatedValue::Unavailable(vec![MissingValue::ForeignAssetRate])
        );
        assert_eq!(
            result.category_activity_clp["food"],
            CalculatedValue::Incomplete {
                known_subtotal: d("100"),
                unresolved: vec![UnresolvedActivity {
                    kind: ActivityKind::Expense,
                    category_id: Some("food".into()),
                    currency: Some("USD".into()),
                    amount: Some(d("2")),
                }]
            }
        );
        assert_eq!(result.unresolved_activity.len(), 1);
        assert!(!result.can_close);
    }

    #[test]
    fn unresolved_only_category_is_present_with_incomplete_activity_and_remaining() {
        let mut input = period_input();
        input.activities.push(TransactionActivity {
            date: date("2026-01-02"),
            kind: ActivityKind::Expense,
            eligible: true,
            category_id: Some("travel".into()),
            amount_clp: None,
            original_currency: Some("USD".into()),
            original_amount: Some(d("10")),
        });

        let result = calculate_period(&input).unwrap();
        let unresolved = vec![UnresolvedActivity {
            kind: ActivityKind::Expense,
            category_id: Some("travel".into()),
            currency: Some("USD".into()),
            amount: Some(d("10")),
        }];
        assert_eq!(
            result.category_activity_clp["travel"],
            CalculatedValue::Incomplete {
                known_subtotal: Decimal::ZERO,
                unresolved: unresolved.clone(),
            }
        );
        assert_eq!(
            result.category_remaining_clp["travel"],
            CalculatedValue::Incomplete {
                known_subtotal: Decimal::ZERO,
                unresolved,
            }
        );
        assert!(!result.can_close);
    }

    #[test]
    fn activity_is_attributed_by_business_date_and_uncategorized_activity_is_visible() {
        let mut input = period_input();
        input.activities.extend([
            TransactionActivity {
                date: date("2026-01-31"),
                kind: ActivityKind::Expense,
                eligible: true,
                category_id: None,
                amount_clp: Some(d("75")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("75")),
            },
            TransactionActivity {
                date: date("2026-02-01"),
                kind: ActivityKind::Expense,
                eligible: true,
                category_id: Some("food".into()),
                amount_clp: Some(d("50")),
                original_currency: Some("CLP".into()),
                original_amount: Some(d("50")),
            },
        ]);
        let result = calculate_period(&input).unwrap();
        assert_eq!(
            result.uncategorized_activity_clp,
            CalculatedValue::Complete(d("75"))
        );
        assert!(result.category_activity_clp.is_empty());
    }

    #[test]
    fn checked_calculation_rejects_negative_assignment_and_overflow() {
        let mut input = period_input();
        input.assignments_clp.insert("food".into(), d("-1"));
        assert_eq!(
            calculate_period(&input),
            Err(CalculationError::NegativeAssignment)
        );
        assert_eq!(
            checked_add(Decimal::MAX, Decimal::ONE),
            Err(CalculationError::ArithmeticOverflow)
        );
    }
}
