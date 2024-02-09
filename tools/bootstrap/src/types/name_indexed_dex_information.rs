#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NameIndexedDexInformation<T> {
    pub ociswap_v1: T,
    pub caviarnine_v1: T,
}

impl<T> NameIndexedDexInformation<T> {
    pub fn map<F, O>(&self, mut map: F) -> NameIndexedDexInformation<O>
    where
        F: FnMut(&T) -> O,
    {
        NameIndexedDexInformation::<O> {
            ociswap_v1: map(&self.ociswap_v1),
            caviarnine_v1: map(&self.caviarnine_v1),
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
            ociswap_v1: map(&self.ociswap_v1)?,
            caviarnine_v1: map(&self.caviarnine_v1)?,
        })
    }
}
