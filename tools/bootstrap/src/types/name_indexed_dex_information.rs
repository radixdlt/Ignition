#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameIndexedDexInformation<T> {
    pub ociswap: T,
    pub caviarnine: T,
}

impl<T> NameIndexedDexInformation<T> {
    pub fn map<F, O>(&self, mut map: F) -> NameIndexedDexInformation<O>
    where
        F: FnMut(&T) -> O,
    {
        NameIndexedDexInformation::<O> {
            ociswap: map(&self.ociswap),
            caviarnine: map(&self.caviarnine),
        }
    }

    pub fn try_map<F, O, E>(
        &self,
        mut map: F,
    ) -> Result<NameIndexedDexInformation<O>, E>
    where
        F: FnMut(&T) -> Result<O, E>,
    {
        Ok(NameIndexedDexInformation::<O> {
            ociswap: map(&self.ociswap)?,
            caviarnine: map(&self.caviarnine)?,
        })
    }
}
