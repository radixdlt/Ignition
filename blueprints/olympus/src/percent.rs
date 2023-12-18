use radix_engine_derive::*;
use scrypto::prelude::*;
use std::ops::*;

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
