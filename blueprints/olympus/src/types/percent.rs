use radix_engine_derive::*;
use scrypto::prelude::*;
use std::ops::*;

use humantime::format_duration;

/// Represents a percentage with the [`Decimal`] as the underlying type used to
/// represent the percentage. This is a value between `0` and `1` where
/// `dec!(0)` is 0% and `dec!(1)` is 100%. This type is checked upon SBOR
/// decoding and construction to ensure that it matches these conditions.
#[derive(
    Clone,
    Copy,
    Debug,
    ScryptoEncode,
    ScryptoCategorize,
    ScryptoDescribe,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[sbor(transparent)]
pub struct Percent(Decimal);

impl Percent {
    pub fn new(value: Decimal) -> Option<Self> {
        if value >= Decimal::ZERO && value <= Decimal::ONE {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn value(&self) -> &Decimal {
        &self.0
    }
}

impl Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0 * 100)
    }
}

impl Deref for Percent {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for Percent
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let inner =
            <Decimal as Decode<
                ScryptoCustomValueKind,
                D,
            >>::decode_body_with_value_kind(decoder, value_kind)?;
        Self::new(inner).ok_or(DecodeError::InvalidCustomValue)
    }
}

/// A type used for the lockup period that can be creates from various time
/// durations and that implements display in the desired way.
#[derive(Clone, Copy, ScryptoSbor, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[sbor(transparent)]
pub struct LockupPeriod(u64);

impl LockupPeriod {
    pub fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub fn from_minutes(minutes: u64) -> Self {
        Self::from_seconds(minutes * 60)
    }

    pub fn from_hours(hours: u64) -> Self {
        Self::from_minutes(hours * 60)
    }

    pub fn from_days(days: u64) -> Self {
        Self::from_hours(days * 24)
    }

    pub fn from_weeks(weeks: u64) -> Self {
        Self::from_days(weeks * 7)
    }

    // One month approx 30.44 days
    pub fn from_months(months: u64) -> Self {
        Self::from_seconds(months * 2_630_016)
    }

    pub fn seconds(&self) -> &u64 {
        &self.0
    }
}

impl Display for LockupPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&format_duration(std::time::Duration::new(self.0, 0)), f)
    }
}

impl Debug for LockupPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} seconds", self.0)
    }
}

impl Deref for LockupPeriod {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
