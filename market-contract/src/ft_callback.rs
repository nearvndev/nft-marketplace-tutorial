use crate::*;
use near_sdk::promise_result_as_success;

//GAS constants to attach to calls
const GAS_FOR_ROYALTIES: Gas = 115_000_000_000_000;
const GAS_FOR_NFT_TRANSFER: Gas = 15_000_000_000_000;
const GAS_FOR_FT_TRANSFER: Gas = 15_000_000_000_000;
//constant used to attach 0 NEAR to a call
const NO_DEPOSIT: Balance = 0;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FTSaleArgs {
    pub nft_contract_id: AccountId,
    pub token_id: TokenId
}

pub trait FungibleTokenReceiver {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft_contract)]
pub trait FungibleTokenCore {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let ft_contract_id = env::predecessor_account_id();

        let FTSaleArgs { nft_contract_id, token_id } = near_sdk::serde_json::from_str(&msg).expect("Not valid FT Sale args");
        let contract_and_token_id: ContractAndTokenId = format!("{}{}{}", nft_contract_id, ".", token_id);
        let sale = self.sales.get(&contract_and_token_id).expect("Not found sale");

        let price = sale.sale_conditions;
        assert!(amount.0 >= price.amount.0, "Amount should be greater than NFT price");
        assert_ne!(sender_id, sale.owner_id, "Can not bid on your own sale");
        assert_eq!(ft_contract_id, price.contract_id, "Payout same token in sale");

        self.ft_process_purchase(
            nft_contract_id, 
            token_id, 
            price, 
            sender_id
        )
    }
}

#[near_bindgen]
impl Contract {
    pub(crate) fn ft_process_purchase(&mut self, nft_contract_id: AccountId, token_id: TokenId, price: SalePrice, buyer_id: AccountId) -> PromiseOrValue<U128> {
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

        ext_nft_contract::nft_transfer_payout(
            buyer_id.clone(), 
            token_id, 
            sale.approval_id, 
            "Payout from market contract".to_string(), 
            price.amount, 
            10, 
            &nft_contract_id, 
            1, 
            GAS_FOR_NFT_TRANSFER
        ).then(ext_self::ft_resolve_purchase(
            buyer_id, 
            price,
            &env::current_account_id(), 
            NO_DEPOSIT, 
            GAS_FOR_ROYALTIES
        )).into()
    }

    pub fn ft_resolve_purchase(&mut self, buyer_id: AccountId, price: SalePrice) -> U128 {
        let payout_option = promise_result_as_success().and_then(| value | {
            let payout_object: Payout = near_sdk::serde_json::from_slice::<Payout>(&value).expect("Invalid payout object");

            if payout_object.payout.len() > 10 || payout_object.payout.is_empty() {
                env::log("Cannot have more than 10 royalities".as_bytes());
                None
            } else {
                let mut remainder = price.amount.0;

                for &value in payout_object.payout.values() {
                    remainder = remainder.checked_sub(value.0)?;
                }

                if remainder == 0 || remainder == 1 {
                    Some(payout_object.payout)
                } else {
                    None
                }
            }
        });

        let payout = if let Some(payout_option) = payout_option {
            payout_option
        } else {
              ext_ft_contract::ft_transfer(
                buyer_id.clone(),
                price.amount,
                None,
                &price.contract_id,
                1,
                GAS_FOR_FT_TRANSFER
            );
            
            return U128(0);
        };

        for (reciver_id, amount) in payout {
            ext_ft_contract::ft_transfer(
                reciver_id.clone(),
                amount,
                None,
                &price.contract_id,
                1,
                GAS_FOR_FT_TRANSFER
            );
        }

        U128(0)
    }
}