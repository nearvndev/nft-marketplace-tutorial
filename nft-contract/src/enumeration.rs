use crate::*;

#[near_bindgen]
impl Contract {
    // Lấy tổng số token đang có trong contract
    pub fn nft_total_supply(&self) -> U128 {
        U128(self.token_metadata_by_id.len() as u128)
    }

    // Lấy tổng số token đang có của account_id
    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);
        if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            U128(tokens_for_owner_set.len() as u128)
        } else {
            U128(0)
        }
    }

    // Lấy danh sách token có paging
    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonToken> {
        let token_keys = self.token_metadata_by_id.keys_as_vector();

        let start = u128::from(from_index.unwrap_or(U128(0)));

        token_keys.iter()
        .skip(start as usize)
        .take(limit.unwrap_or(0) as usize)
        .map(|token_id| self.nft_token(token_id.clone()).unwrap() )
        .collect()
    }

    // Lấy danh sach token của account_id
    pub fn nft_tokens_for_owner(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonToken> {
        let token_keys = self.tokens_per_owner.get(&account_id);

        let keys = if let Some(token_keys) = token_keys {
            token_keys
        } else {
            return vec![];
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));

        keys.as_vector()
        .iter()
        .skip(start as usize)
        .take(limit.unwrap_or(0) as usize)
        .map(|token_id| self.nft_token(token_id.clone()).unwrap() )
        .collect()
    }
}