#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameIndexedResourceInformation<T> {
    pub bitcoin: T,
    pub ethereum: T,
    pub usdc: T,
    pub usdt: T,
}

impl<T> NameIndexedResourceInformation<T> {
    pub fn map<F, O>(&self, mut map: F) -> NameIndexedResourceInformation<O>
    where
        F: FnMut(&T) -> O,
    {
        NameIndexedResourceInformation::<O> {
            bitcoin: map(&self.bitcoin),
            ethereum: map(&self.ethereum),
            usdc: map(&self.usdc),
            usdt: map(&self.usdt),
        }
    }

    pub fn try_map<F, O, E>(
        &self,
        mut map: F,
    ) -> Result<NameIndexedResourceInformation<O>, E>
    where
        F: FnMut(&T) -> Result<O, E>,
    {
        Ok(NameIndexedResourceInformation::<O> {
            bitcoin: map(&self.bitcoin)?,
            ethereum: map(&self.ethereum)?,
            usdc: map(&self.usdc)?,
            usdt: map(&self.usdt)?,
        })
    }
}

impl<T> std::iter::IntoIterator for NameIndexedResourceInformation<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.bitcoin, self.ethereum, self.usdc, self.usdt].into_iter()
    }
}
