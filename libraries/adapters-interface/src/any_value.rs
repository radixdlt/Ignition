use radix_engine_interface::prelude::*;

#[derive(Clone, Debug, Sbor, PartialEq, Eq)]
#[sbor(transparent)]
pub struct AnyValue(Vec<u8>);

impl AnyValue {
    pub fn from_typed<T>(typed: &T) -> Result<Self, AnyValueError>
    where
        T: ScryptoEncode,
    {
        scrypto_encode(typed).map(Self).map_err(Into::into)
    }

    pub fn as_typed<T>(&self) -> Result<T, AnyValueError>
    where
        T: ScryptoDecode,
    {
        scrypto_decode(&self.0).map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub enum AnyValueError {
    EncodeError(EncodeError),
    DecodeError(DecodeError),
}

impl From<EncodeError> for AnyValueError {
    fn from(value: EncodeError) -> Self {
        Self::EncodeError(value)
    }
}

impl From<DecodeError> for AnyValueError {
    fn from(value: DecodeError) -> Self {
        Self::DecodeError(value)
    }
}

#[cfg(test)]
mod test {
    use super::AnyValue;

    #[test]
    fn simple_roundtrip_test() {
        // Arrange
        let value = 12;

        // Act
        let any_value = AnyValue::from_typed(&value).unwrap();

        // Assert
        assert_eq!(any_value.as_typed::<i32>().unwrap(), value)
    }
}