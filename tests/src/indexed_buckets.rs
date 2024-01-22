use crate::prelude::*;

/// Buckets indexed and aggregated by the resource address.
pub struct IndexedBuckets(IndexMap<ResourceAddress, Bucket>);

impl IndexedBuckets {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn from_bucket<Y, E>(
        bucket: impl Into<Bucket>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let mut this = Self::new();
        this.insert(bucket, api)?;
        Ok(this)
    }

    pub fn from_buckets<Y, E>(
        buckets: impl IntoIterator<Item = impl Into<Bucket>>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let mut this = Self::new();

        for bucket in buckets.into_iter() {
            this.insert(bucket, api)?;
        }

        Ok(this)
    }

    pub fn get(&self, resource_address: &ResourceAddress) -> Option<&Bucket> {
        self.0.get(resource_address)
    }

    pub fn get_mut(
        &mut self,
        resource_address: &ResourceAddress,
    ) -> Option<&mut Bucket> {
        self.0.get_mut(resource_address)
    }

    pub fn insert<Y, E>(
        &mut self,
        bucket: impl Into<Bucket>,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let bucket = bucket.into();
        let resource_address = bucket.resource_address(api)?;
        if let Some(existing_bucket) = self.0.get_mut(&resource_address) {
            existing_bucket.put(bucket, api)?;
        } else {
            self.0.insert(resource_address, bucket);
        };
        Ok(())
    }

    pub fn keys(&self) -> impl Iterator<Item = &ResourceAddress> {
        self.0.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Bucket> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Bucket> {
        self.0.values_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Default for IndexedBuckets {
    fn default() -> Self {
        Self::new()
    }
}
