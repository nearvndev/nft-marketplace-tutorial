use std::{mem::size_of, collections::HashMap};

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

pub(crate) fn assert_at_least_one_yocto() {
    assert!(env::attached_deposit() >= 1,
    "Require attached deposit of at least 1 yoctoNear")
}

pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}

pub(crate) fn refund_approved_account_ids_iter<'a, I>(sender_id: AccountId, approved_account_ids: I)
    where 
        I: Iterator<Item = &'a AccountId>,
{
    let storage_released: u64 = approved_account_ids.map(bytes_for_approved_account_id).sum();

    Promise::new(sender_id).transfer(Balance::from(storage_released) * env::storage_byte_cost());
}

pub(crate) fn refund_approved_account_ids(sender_id: AccountId, approved_account_ids: &HashMap<AccountId, u64>) {
    refund_approved_account_ids_iter(sender_id, approved_account_ids.keys());
}

pub(crate) fn royalty_to_payout(royalty_percentage: u32, amount_to_pay: Balance) -> U128 {
    U128(royalty_percentage as u128 * amount_to_pay / 10_000u128)
}