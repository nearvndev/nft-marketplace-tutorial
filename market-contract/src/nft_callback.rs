use crate::*;

pub trait NonFungibleTokenApprovalReceiver {
    fn nft_on_approve(&mut self, token_id: TokenId, owner_id: AccountId, approval_id: u64, msg: String);
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    pub sale_conditions: SalePrice
}


#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Contract {
    fn nft_on_approve(&mut self, token_id: TokenId, owner_id: AccountId, approval_id: u64, msg: String) {
        // User => NFT contract => Market Contract
        // Signer account => Predecessor account => Current account

        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();

        assert_ne!(nft_contract_id, signer_id, "nft_on_approve should only called via cross contract call");
        assert_eq!(signer_id, owner_id, "owner_id should be signer_id");

        // Check cover storage

        let storage_balance = self.storage_deposit.get(&signer_id).unwrap_or(0);
        let storage_minimum_amount = self.storage_minimum_balance().0;
        let storage_required = (self.get_supply_by_owner_id(signer_id.clone()).0 + 1) * storage_minimum_amount;

        assert!(storage_balance >= storage_required, "Storage balance not enough for cover storage staking");

        let SaleArgs { sale_conditions } = near_sdk::serde_json::from_str(&msg).expect("Not valid Sale Args");

        let contract_and_token_id = format!("{}{}{}", nft_contract_id.clone(), ".", token_id.clone());

        // Thêm sales collections
        self.sales.insert(&contract_and_token_id, &Sale {
            owner_id: owner_id.clone(),
            approval_id,
            nft_contract_id: nft_contract_id.clone(),
            token_id: token_id.clone(),
            sale_conditions
        });

        let mut by_owner_id = self.by_owner_id.get(&owner_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::InnterByOwnerIdKey {
                    account_id_hash: hash_account_id(&owner_id)
                }.try_to_vec().unwrap()
            )
        });

        by_owner_id.insert(&contract_and_token_id);
        self.by_owner_id.insert(&owner_id, &by_owner_id);

        let mut by_contract_id = self.by_contract_id.get(&nft_contract_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::InnerByContractIdKey {
                    account_id_hash: hash_account_id(&nft_contract_id)
                }.try_to_vec().unwrap()
            )
        });

        by_contract_id.insert(&token_id);
        self.by_contract_id.insert(&nft_contract_id, &by_contract_id);
    }
}