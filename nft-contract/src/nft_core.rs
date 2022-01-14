use near_sdk::env::log;

use crate::*;

const GAS_FOR_RESOLVE_TRANSFER: Gas = 10_000_000_000_000;
const GAS_FOR_NFT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;
const NO_DEPOSIT: Balance = 0;

pub trait NonFungibleTokenCore {
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>);

    // return true nếu transfer NFT được thực hiện thành công
    fn nft_transfer_call(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>, msg: String) -> PromiseOrValue<bool>;
}

#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    // Method lưu trên Contract B, A thực cross contract call nft_on_transfer
    // return true nếu như NFT cần được rollback lại cho owner cũ
    fn nft_on_transfer(&mut self, sender_id: AccountId, previous_owner_id: AccountId, token_id: TokenId, msg: String) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    // Nếu contract B yêu cầu rollback lại cho owner cũ => A sẽ rollback lại data trong nft_resolve_transfer
    fn nft_resolve_transfer(&mut self, owner_id: AccountId, receiver_id: AccountId, token_id: TokenId) -> bool;
}

trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(&mut self, owner_id: AccountId, receiver_id: AccountId, token_id: TokenId) -> bool;
}
#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    // Yêu cầu deposit 1 yoctoNear để bảo mật cho user
    #[payable]
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        self.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            memo
        );
    }

    #[payable]
    fn nft_transfer_call(&mut self, receiver_id: AccountId , token_id: TokenId, memo: Option<String>, msg: String) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let previous_token = self.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            memo
        );

        ext_non_fungible_token_receiver::nft_on_transfer(
            sender_id.clone(), 
            previous_token.owner_id.clone(), 
            token_id.clone(), 
            msg, 
            &receiver_id, 
            NO_DEPOSIT, 
            env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL
        ).then(ext_self::nft_resolve_transfer(
            previous_token.owner_id, 
            receiver_id, 
            token_id, 
            &env::current_account_id(), 
            NO_DEPOSIT, 
        GAS_FOR_RESOLVE_TRANSFER
        )).into()
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    fn nft_resolve_transfer(&mut self, owner_id: AccountId, receiver_id: AccountId, token_id: TokenId) -> bool {
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            if let Ok(is_rollback_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
                return !is_rollback_token;
            }
        }

        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            if token.owner_id != receiver_id {
                return true;
            }
            token
        } else {
            return true;
        };

        log!("Rollback {} from @{} to @{}", token_id, receiver_id, owner_id);
    
        self.internal_remove_token_from_owner(&token_id, &receiver_id);
        self.internal_add_token_to_owner(&token_id, &owner_id);

        token.owner_id = owner_id;

        self.tokens_by_id.insert(&token_id, &token);

        false
    }
}