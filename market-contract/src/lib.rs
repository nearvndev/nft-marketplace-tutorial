use near_sdk::{AccountId, near_bindgen, PanicOnDefault, Balance, env, Promise, CryptoHash, ext_contract, Gas};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::{LookupMap, UnorderedSet, UnorderedMap};

pub use crate::sale_view::*;
pub use crate::utils::*;
pub use crate::nft_callback::*;
pub use crate::sale::*;

const STORAGE_PER_SALE: u128 = 1000 * env::STORAGE_PRICE_PER_BYTE;

mod sale_view;
mod utils;
mod sale;
mod nft_callback;
mod internal;

pub type TokenId = String;
pub type NFTContractId = String;
pub type SalePriceInYoctoNear = U128;
pub type ContractAndTokenId = String; //nft-tutorial.vbidev.testnet.VBI_NFT#01

#[derive(Deserialize, Serialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: NFTContractId,
    pub token_id: TokenId,
    pub sale_conditions: SalePriceInYoctoNear
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // Owner of contract
    pub owner_id: AccountId,

    // Sales của token
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,

    // Danh sách sales theo account id
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,

    // Danh sách token_id đang được đăng bán của 1 nft contract
    pub by_contract_id: LookupMap<NFTContractId, UnorderedSet<TokenId>>,

    // Danh sách account deposit để cover storage
    pub storage_deposit: LookupMap<AccountId, Balance>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum StorageKey {
    SaleKey,
    ByOwnerIdKey,
    InnterByOwnerIdKey {
        account_id_hash: CryptoHash
    },
    ByContractIdKey,
    InnerByContractIdKey {
        account_id_hash: CryptoHash
    },
    StorageDepositKey
}

#[near_bindgen]
impl Contract {

    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            sales: UnorderedMap::new(StorageKey::SaleKey.try_to_vec().unwrap()),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerIdKey.try_to_vec().unwrap()),
            by_contract_id: LookupMap::new(StorageKey::ByContractIdKey.try_to_vec().unwrap()),
            storage_deposit: LookupMap::new(StorageKey::StorageDepositKey.try_to_vec().unwrap())
        }
    }

    #[payable]
    pub fn storate_deposit(&mut self, account_id: Option<AccountId>) {
        let storage_account_id = account_id.unwrap_or(env::predecessor_account_id());
        let deposit = env::attached_deposit();

        assert!(deposit >= STORAGE_PER_SALE, "Requires deposit minimum of {}", STORAGE_PER_SALE);

        let mut balance = self.storage_deposit.get(&storage_account_id).unwrap_or(0);
        balance += deposit;

        self.storage_deposit.insert(&storage_account_id, &balance);
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();

        let amount = self.storage_deposit.remove(&owner_id).unwrap_or(0);
        let sales = self.by_owner_id.get(&owner_id);
        
        let len = sales.map(| s | s.len()).unwrap_or_default();

        let storage_required = u128::from(len) * STORAGE_PER_SALE;

        assert!(amount >= storage_required);

        let diff = amount - storage_required;

        if diff > 0 {
            Promise::new(owner_id.clone()).transfer(diff);
        }

        if storage_required > 0 {
            self.storage_deposit.insert(&owner_id, &storage_required);
        }
    }

    pub fn storage_minimum_balance(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    pub fn storage_balance_of(&self, account_id: Option<AccountId>) -> U128 {
        let owner_id = account_id.unwrap_or(env::predecessor_account_id());

        U128(self.storage_deposit.get(&owner_id).unwrap_or(0))
    }
}