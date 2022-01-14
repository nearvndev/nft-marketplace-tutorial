use crate::*;

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

pub(crate) fn refund_deposit(storage_used: u64) {
    // Tính lượng tiền cần nạp để cover storage
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNear to cover storage", required_cost
    );

    let refund = attached_deposit - required_cost;

    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

pub(crate) fn assert_one_yocto() {
    assert_eq!(env::attached_deposit(), 1,
    "Require attached deposit of exactly 1 yoctoNear")
}