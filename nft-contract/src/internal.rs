use std::collections::HashMap;

use crate::*;

#[near_bindgen]
impl Contract {
    pub(crate) fn internal_add_token_to_owner(&mut self, token_id: &TokenId, account_id: &AccountId) {

        // Nếu account_id đã có ds token rồi, thì sẽ lấy ds token đang có
        // Nếu account_id chưa có trong tokens_per_owner thì tạo mới tokens_Set
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(StorageKey::TokenPerOwnerInnerKey {
                account_id_hash: hash_account_id(account_id)
            }.try_to_vec().unwrap())
        });

        tokens_set.insert(token_id);

        self.tokens_per_owner.insert(account_id, &tokens_set);
    }

    pub(crate) fn internal_remove_token_from_owner(&mut self, token_id: &TokenId, account_id: &AccountId) {
        let mut tokens_set = self.tokens_per_owner.get(account_id).expect("Token should be owned by sender");

        tokens_set.remove(token_id);
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        } else {
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }
    }

    // return lại data token cũ trước khi thực hiện transfer
    /**
     * - Kiểm tra token_id có tồn tại không?
     * - sender_id có phải là owner của token không?
     * - sender_id và receiver_id trùng nhau không?
     * - Xoá token khỏi owner cũ
     * - Thêm token cho receiver_id
     */
    pub(crate) fn internal_transfer(
        &mut self, 
        sender_id: &AccountId, 
        receiver_id: &AccountId, 
        token_id: &TokenId, 
        approval_id: Option<u64>,
        memo: Option<String>) -> Token {
            let token = self.tokens_by_id.get(token_id).expect("Not found token");
            // check owner
            if sender_id != &token.owner_id {
                if !token.approved_account_ids.contains_key(sender_id) {
                    env::panic("Sender must be the token owner".as_bytes());
                }

                if let Some(enforced_approval_id) = approval_id {
                    let actual_approval_id = token.approved_account_ids.get(sender_id).expect("Sender is not approved account");

                    assert_eq!(
                        actual_approval_id, &enforced_approval_id,
                        "The actual approval id {} is different from the given approval id {}",
                        actual_approval_id, enforced_approval_id
                    )
                }
            };

            assert_ne!(&token.owner_id, receiver_id, "The token owner and the receiver should be different");

            self.internal_remove_token_from_owner(&token_id, &token.owner_id);
            self.internal_add_token_to_owner(&token_id, receiver_id);

            let new_token = Token {
                owner_id: receiver_id.clone(),
                approved_account_ids: HashMap::default(),
                next_approval_id: token.next_approval_id,
                royalty: token.royalty.clone()
            };

            self.tokens_by_id.insert(token_id,&new_token);

            if let Some(memo) = memo.as_ref() {
                log!("Memo {}", memo);
            };

            // NFT TRANSFER LOG
            let mut authorized_id = None;
            if approval_id.is_some() {
                authorized_id = Some(sender_id.to_string());
            }

            let nft_transfer_log: EventLog = EventLog {
                standard: "nep171".to_string(),
                version: "1.0.0".to_string(),
                event: EventLogVariant::NftTransfer(vec![ NftTransferLog {
                    authorized_id,
                    old_owner_id: token.owner_id.to_string(),
                    new_owner_id: receiver_id.to_string(),
                    token_ids: vec![token_id.to_string()],
                    memo
                } ])
            };

            env::log(&nft_transfer_log.to_string().as_bytes());

            token
    }
}