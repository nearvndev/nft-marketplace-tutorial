use crate::*;
/// Assert that 1 yoctoNEAR was attached.
pub fn assert_one_yocto() {
    assert!(env::attached_deposit() == 1, "Requires attached deposit of exactly 1 yoctoNEAR")
}

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}