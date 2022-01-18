use std::collections::HashMap;

use near_sdk::env;

use crate::*;

#[near_bindgen]
impl Contract {
    /**
     * - Yêu cầu user nạp tiền để cover phí lưu trữ
     * - Thêm token vào token_by_id
     * - Thêm token metadata
     * - Thêm token vào ds sở hữu bởi owner
     * - Refund lại NEAR user deposit thừa
     */
    #[payable]
    pub fn nft_mint(&mut self, token_id: TokenId, metadata: TokenMetadata, receiver_id: AccountId, perpetual_royalties: Option<HashMap<AccountId, u32>>) {
        let before_storage_usage = env::storage_usage();

        let mut royalty = HashMap::new();

        // if perpetual royalties were passed into the function: 
        if let Some(perpetual_royalties) = perpetual_royalties {
            //make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

            //iterate through the perpetual royalties and insert the account and amount in the royalty map
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        let token = Token {
            owner_id: receiver_id,
            approved_account_ids: HashMap::default(),
            next_approval_id: 0,
            royalty
        };

        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exsits"
        );

        self.token_metadata_by_id.insert(&token_id, &metadata);

        // set token per owner
        self.internal_add_token_to_owner(&token_id, &token.owner_id);

        // NFT MINT LOG
        let nft_mint_log: EventLog = EventLog {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::NftMint(vec![ NftMintLog {
                owner_id: token.owner_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo: None
            } ])
        };
        env::log(&nft_mint_log.to_string().as_bytes());

        let after_storage_usage = env::storage_usage();
        // Refund near
        refund_deposit(after_storage_usage - before_storage_usage);
    }

    pub fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        let token = self.tokens_by_id.get(&token_id);

        if let Some(token) = token {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();

            Some(JsonToken {
                owner_id: token.owner_id,
                token_id,
                metadata,
                approved_account_ids: token.approved_account_ids,
                royalty: token.royalty
            })
        } else {
            None
        }
    }
}