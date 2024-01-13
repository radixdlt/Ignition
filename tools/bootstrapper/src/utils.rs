use rand::Rng;

#[inline]
pub fn random_nonce() -> u32 {
    rand::thread_rng().gen()
}
