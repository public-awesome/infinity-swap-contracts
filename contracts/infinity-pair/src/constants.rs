pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Top level storage key. Values must not conflict.
/// Each key is only one byte long to ensure we use the smallest possible storage keys.
#[repr(u8)]
pub enum TopKey {
    InfinityGlobal = b'G',
    NftDeposits = b'N',
    PairImmutable = b'P',
    PairConfig = b'C',
    PairInternal = b'I',
}

impl TopKey {
    pub const fn as_str(&self) -> &str {
        let array_ref = unsafe { std::mem::transmute::<_, &[u8; 1]>(self) };
        match core::str::from_utf8(array_ref) {
            Ok(a) => a,
            Err(_) => panic!("Non-utf8 enum value found. Use a-z, A-Z and 0-9"),
        }
    }
}
