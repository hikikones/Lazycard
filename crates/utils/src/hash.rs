use std::hash::{Hash, Hasher};

pub fn hash_fast(input: impl AsRef<[u8]>) -> u64 {
    let mut hasher = ahash::AHasher::default();
    input.as_ref().hash(&mut hasher);
    hasher.finish()
}

pub fn hash_portable(input: impl AsRef<[u8]>) -> u64 {
    seahash::hash(input.as_ref())
}
