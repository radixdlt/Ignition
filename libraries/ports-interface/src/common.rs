use radix_engine_derive::*;
use scrypto::prelude::*;

/// A price type representing the price in terms of some base and quote assets.
///
/// The information of the base and quote assets is captured by this type to
/// make certain calculations easier and to also allow for some checks to make
/// sure that an incorrect price is not used for calculations.
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
            Some(
                (other.price - self.price)
                    .checked_abs()
                    .unwrap_or(Decimal::MAX)
                    / self.price,
            )
        } else if self.base == other.quote && self.quote == other.base {
            self.relative_difference(&other.reciprocal())
        } else {
            None
        }
    }

    /// Computes the output amount based on some input amount.
    ///
    /// This method calculates the output if an exchange happens at this price
    /// where the output has a [`ResourceAddress`] to make it clear what unit
    /// the output is in. If a resource that is neither the base nor the quote
    /// is passed then [`None`] is returned.
    ///
    /// In a price that is BASE/QUOTE the unit is QUOTE/BASE. Therefore, if the
    /// base tokens are passed, their amount is multiplied by the price and then
    /// returned. If the quote tokens are passed, the reciprocal of the price is
    /// multiplied by the passed amount. Otherwise, the resource does not belong
    /// to this price.
    ///
    /// # Arguments
    ///
    /// `resource_address`: [`ResourceAddress`] - The address of the input
    /// resources.
    /// `amount`: [`Decimal`] - The amount of the input resources.
    ///
    /// # Returns
    ///
    /// [`Option<(ResourceAddress, Decimal)>`] - The address and amount of the
    /// output if the input resource is either the base or the quote asset. Else
    /// this is [`None`].
    pub fn exchange(
        &self,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> Option<(ResourceAddress, Decimal)> {
        if resource_address == self.base {
            Some((self.quote, self.price * amount))
        } else if resource_address == self.quote {
            Some((self.base, amount / self.price))
        } else {
            None
        }
    }

    /// Computes the reciprocal of the address.
    ///
    /// This method computes the price's reciprocal by exchanging the base with
    /// the quote and diving 1 by the price.
    ///
    /// # Returns
    ///
    /// [`Price`] - The reciprocal of the price.
    pub fn reciprocal(&self) -> Price {
        Price {
            base: self.quote,
            quote: self.base,
            price: 1 / self.price,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const BITCOIN: ResourceAddress = ResourceAddress::new_or_panic([
        93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 1,
    ]);
    const USD: ResourceAddress = ResourceAddress::new_or_panic([
        93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 2,
    ]);

    #[test]
    fn percentage_difference_with_opposite_base_and_quote_is_calculated_correctly(
    ) {
        // Arrange
        let p1 = Price {
            base: BITCOIN,
            quote: USD,
            price: dec!(100),
        };
        let p2 = Price {
            base: USD,
            quote: BITCOIN,
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
            base: BITCOIN,
            quote: USD,
            price: dec!(100),
        };
        let p2 = Price {
            base: BITCOIN,
            quote: USD,
            price: dec!(50),
        };

        // Act
        let difference = p1.relative_difference(&p2).unwrap();

        // Assert
        assert_eq!(difference, dec!(0.5))
    }

    #[test]
    fn exchange_method_calculates_as_expected1() {
        // Arrange
        let btc = BITCOIN;
        let usd = USD;

        let price = Price {
            base: btc,
            quote: usd,
            price: dec!(43000),
        };

        // Act
        let (out_address, out_amount) = price.exchange(btc, dec!(1)).unwrap();

        // Assert
        assert_eq!(out_address, usd);
        assert_eq!(out_amount, dec!(43000));
    }

    #[test]
    fn exchange_method_calculates_as_expected2() {
        // Arrange
        let btc = BITCOIN;
        let usd = USD;

        let price = Price {
            base: btc,
            quote: usd,
            price: dec!(43000),
        };

        // Act
        let (out_address, out_amount) =
            price.exchange(usd, dec!(43000)).unwrap();

        // Assert
        assert_eq!(out_address, btc);
        assert_eq!(out_amount, dec!(1));
    }

    #[test]
    fn price_reciprocal_is_what_we_expect_it_to_be() {
        // Arrange
        let btc = BITCOIN;
        let usd = USD;

        let price = Price {
            base: btc,
            quote: usd,
            price: dec!(2),
        };

        // Act
        let reciprocal = price.reciprocal();

        // Assert
        assert_eq!(reciprocal.base, price.quote);
        assert_eq!(reciprocal.quote, price.base);
        assert_eq!(reciprocal.price, dec!(1) / price.price);
    }
}
