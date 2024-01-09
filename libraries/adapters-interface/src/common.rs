use radix_engine_derive::*;
use scrypto::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ScryptoSbor)]
pub struct Price {
    pub base: ResourceAddress,
    pub quote: ResourceAddress,
    pub price: Decimal,
}

impl Price {
    /// Computes the difference ratio between `self` and `other`.
    ///
    /// Attempts to compute the difference ratio between the two prices:
    /// `self` and `other`. If the base and the quote are different then a
    /// reciprocal of the price is used to calculate the difference. A [`None`]
    /// is returned if `other` has a base or quote resource address that is
    /// neither the base nor the quote of `self`.
    ///
    /// The equation used for the ratio calculation is obtained from this
    /// [article] can is provided below:
    ///
    /// ```math
    /// ratio = |other.price - self.price| / self.price
    /// ```
    ///
    /// # Arguments
    ///
    /// * `other`: [`&Self`] - A reference to another [`Price`] object to
    /// compute the difference ratio between.
    ///
    /// # Returns:
    ///
    /// [`Option<Decimal>`] - An optional [`Decimal`] value is returned which is
    /// in the range [0, ∞] which is of the difference ratio and not percentage
    /// and thus, it is not multiplied by 100. This means that a return of 0
    /// indicates no difference, a return of 1 indicated 100% difference, and
    /// so on. If [`None`] is returned then these two prices where of two
    /// different pairs.
    ///
    /// [article]: https://en.wikipedia.org/wiki/Relative_change
    pub fn relative_difference(&self, other: &Self) -> Option<Decimal> {
        if self.base == other.base && self.quote == other.quote {
            Some((other.price - self.price).checked_abs().unwrap() / self.price)
        } else if self.base == other.quote && self.quote == other.base {
            self.relative_difference(&other.reciprocal())
        } else {
            None
        }
    }

    pub fn exchange(
        &self,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> Option<(ResourceAddress, Decimal)> {
        if resource_address == self.base {
            Some((self.quote, self.price * amount))
        } else if resource_address == self.quote {
            self.reciprocal().exchange(resource_address, amount)
        } else {
            None
        }
    }

    pub fn reciprocal(&self) -> Price {
        Price {
            base: self.quote,
            quote: self.base,
            price: 1 / self.price,
        }
    }
}

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

impl std::ops::Deref for Percent {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn percentage_difference_with_opposite_base_and_quote_is_calculated_correctly(
    ) {
        // Arrange
        let p1 = Price {
            base: ACCOUNT_OWNER_BADGE,
            quote: VALIDATOR_OWNER_BADGE,
            price: dec!(100),
        };
        let p2 = Price {
            base: VALIDATOR_OWNER_BADGE,
            quote: ACCOUNT_OWNER_BADGE,
            price: dec!(1) / dec!(100),
        };

        // Act
        let difference = p1.relative_difference(&p2).unwrap();

        // Assert
        assert_eq!(difference, dec!(0))
    }

    #[test]
    fn simple_percentage_difference_is_calculated_correctly() {
        // Arrange
        let p1 = Price {
            base: ACCOUNT_OWNER_BADGE,
            quote: VALIDATOR_OWNER_BADGE,
            price: dec!(100),
        };
        let p2 = Price {
            base: ACCOUNT_OWNER_BADGE,
            quote: VALIDATOR_OWNER_BADGE,
            price: dec!(50),
        };

        // Act
        let difference = p1.relative_difference(&p2).unwrap();

        // Assert
        assert_eq!(difference, dec!(0.5))
    }
}
