//! Exact wire types for monetary decimal strings and date-only business values.

use std::{fmt, str::FromStr};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

const MAX_DECIMAL_MANTISSA: &str = "79228162514264337593543950335";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum FinancialValueError {
    #[error("must be a plain ASCII decimal string")]
    InvalidDecimalSyntax,
    #[error("decimal value is outside the supported precision envelope")]
    DecimalOutOfRange,
    #[error("must be a date in YYYY-MM-DD format")]
    InvalidBusinessDate,
}

/// A `Decimal` accepted by the first-slice 96-bit, scale 0–28 contract.
///
/// Values are serialized as strings so JSON consumers never pass money through
/// a binary floating-point representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MoneyAmount(Decimal);

impl MoneyAmount {
    pub fn new(value: Decimal) -> Result<Self, FinancialValueError> {
        if value.scale() > 28 {
            return Err(FinancialValueError::DecimalOutOfRange);
        }
        Ok(Self(value))
    }

    pub fn decimal(self) -> Decimal {
        self.0
    }

    pub fn parse(value: &str) -> Result<Self, FinancialValueError> {
        if !is_plain_decimal(value) {
            return Err(FinancialValueError::InvalidDecimalSyntax);
        }

        validate_decimal_envelope(value)?;

        let decimal =
            Decimal::from_str(value).map_err(|_| FinancialValueError::DecimalOutOfRange)?;
        Self::new(decimal)
    }
}

fn validate_decimal_envelope(value: &str) -> Result<(), FinancialValueError> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if fraction.len() > 28 {
        return Err(FinancialValueError::DecimalOutOfRange);
    }

    // Check the scaled mantissa as text before parsing. Decimal's parser may
    // otherwise round a value with more than 28 fractional digits.
    let digits = format!("{whole}{fraction}");
    let significant = digits.trim_start_matches('0');
    if significant.len() > MAX_DECIMAL_MANTISSA.len()
        || (significant.len() == MAX_DECIMAL_MANTISSA.len() && significant > MAX_DECIMAL_MANTISSA)
    {
        return Err(FinancialValueError::DecimalOutOfRange);
    }

    Ok(())
}

impl fmt::Display for MoneyAmount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for MoneyAmount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MoneyAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}

fn is_plain_decimal(value: &str) -> bool {
    let value = value.strip_prefix('-').unwrap_or(value);
    if value.is_empty() {
        return false;
    }

    let mut parts = value.split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some()
        || whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
    {
        return false;
    }

    match fraction {
        Some(digits) => !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()),
        None => true,
    }
}

/// A calendar date used as a financial business date, independent of timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BusinessDate(NaiveDate);

impl BusinessDate {
    pub fn new(value: NaiveDate) -> Self {
        Self(value)
    }

    pub fn date(self) -> NaiveDate {
        self.0
    }

    pub fn parse(value: &str) -> Result<Self, FinancialValueError> {
        let bytes = value.as_bytes();
        let shape_matches = bytes.len() == 10
            && bytes[0..4].iter().all(u8::is_ascii_digit)
            && bytes[4] == b'-'
            && bytes[5..7].iter().all(u8::is_ascii_digit)
            && bytes[7] == b'-'
            && bytes[8..10].iter().all(u8::is_ascii_digit);
        if !shape_matches {
            return Err(FinancialValueError::InvalidBusinessDate);
        }

        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| FinancialValueError::InvalidBusinessDate)
    }
}

impl fmt::Display for BusinessDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.format("%Y-%m-%d").fmt(formatter)
    }
}

impl Serialize for BusinessDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BusinessDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use super::{BusinessDate, FinancialValueError, MoneyAmount};

    #[test]
    fn decimal_parser_preserves_supported_precision_and_signed_values() {
        for input in [
            "0",
            "1250.50",
            "0.0000000000000000000000000001",
            "79228162514264337593543950335",
        ] {
            let parsed = MoneyAmount::parse(input).unwrap();
            assert_eq!(parsed.to_string(), input);
            assert_eq!(
                serde_json::to_string(&parsed).unwrap(),
                format!("\"{input}\"")
            );
        }

        let negative_zero = MoneyAmount::parse("-0.00").unwrap();
        assert_eq!(negative_zero.decimal(), Decimal::ZERO);
        assert_eq!(negative_zero.to_string(), "0.00");

        let negative = MoneyAmount::parse("-79228162514264337593543950335").unwrap();
        assert!(negative.decimal().is_sign_negative());
    }

    #[test]
    fn decimal_parser_rejects_invalid_wire_values_and_envelope_overflow() {
        for input in ["", ".5", "1.", "+1", " 1", "1 ", "1,000", "1e3", "NaN", "∞"] {
            assert_eq!(
                MoneyAmount::parse(input),
                Err(FinancialValueError::InvalidDecimalSyntax),
                "accepted {input:?}"
            );
        }
        assert_eq!(
            MoneyAmount::parse("79228162514264337593543950336"),
            Err(FinancialValueError::DecimalOutOfRange)
        );
        assert_eq!(
            MoneyAmount::parse("7922816251426433759354395033.6"),
            Err(FinancialValueError::DecimalOutOfRange)
        );
        assert_eq!(
            MoneyAmount::parse("0.00000000000000000000000000001"),
            Err(FinancialValueError::DecimalOutOfRange)
        );
        assert!(serde_json::from_str::<MoneyAmount>("1.25").is_err());
    }

    #[test]
    fn decimal_type_checks_scale_and_magnitude_from_existing_values() {
        assert_eq!(
            MoneyAmount::new(Decimal::MAX),
            Ok(MoneyAmount::parse("79228162514264337593543950335").unwrap())
        );
    }

    #[test]
    fn date_parser_accepts_only_valid_zero_padded_calendar_dates() {
        let date = BusinessDate::parse("2026-11-05").unwrap();
        assert_eq!(date.date(), NaiveDate::from_ymd_opt(2026, 11, 5).unwrap());
        assert_eq!(serde_json::to_string(&date).unwrap(), "\"2026-11-05\"");

        for input in [
            "2026-1-05",
            "2026-11-5",
            "2026/11/05",
            "2026-02-30",
            "2026-11-05T00:00:00",
            " 2026-11-05",
        ] {
            assert_eq!(
                BusinessDate::parse(input),
                Err(FinancialValueError::InvalidBusinessDate),
                "accepted {input:?}"
            );
        }
        assert!(serde_json::from_str::<BusinessDate>("2026-11-05").is_err());
    }
}
