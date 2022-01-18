use crate::*;

#[near_bindgen]
impl Contract {
    // Lấy tổng số sale đang đăng bán trên market
    pub fn get_supply_sales(&self) -> U128 {
        U128(self.sales.len() as u128)
    }

    // Lấy tổng số sale đang đăng bán theo owner_id
    pub fn get_supply_by_owner_id(&self, account_id: AccountId) -> U128 {
        let sales_by_owner_id = self.by_owner_id.get(&account_id);
        if let Some(sales_by_owner_id) = sales_by_owner_id {
            U128(sales_by_owner_id.len() as u128)
        } else {
            U128(0)
        }
    }

    // Lấy tổng số sale đang đăng bán theo contract_id
    pub fn get_suppy_by_contract_id(&self, contract_id: NFTContractId) -> U128 {
        let tokens_by_contract_id = self.by_contract_id.get(&contract_id);
        if let Some(tokens_by_contract_id) = tokens_by_contract_id {
            U128(tokens_by_contract_id.len() as u128)
        } else {
            U128(0)
        }
    }

    pub fn get_sales(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<Sale> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

         self.sales.values()
         .skip(start as usize)
         .take(limit.unwrap_or(0) as usize)
         .collect()
    }

    pub fn get_sales_by_owner_id(&self, account_id: AccountId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Sale> {
        let by_owner_id = self.by_owner_id.get(&account_id);
        let contract_token_ids = if let Some(by_owner_id) = by_owner_id {
            by_owner_id
        } else {
            return vec![];
        };
        let start = u128::from(from_index.unwrap_or(U128(0)));

        contract_token_ids.as_vector()
        .iter()
        .skip(start as usize)
        .take(limit.unwrap_or(0) as usize)
        .map( | contract_token_id | self.sales.get(&contract_token_id).unwrap() )
        .collect()
    }

    pub fn get_sales_by_contract_id(&self, contract_id: NFTContractId, from_index: Option<U128>, limit: Option<u64>) -> Vec<Sale> {
        let tokens_by_contract_id = self.by_contract_id.get(&contract_id);

        let token_ids = if let Some(tokens_by_contract_id) = tokens_by_contract_id {
            tokens_by_contract_id
        } else {
            return vec![];
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));
        token_ids.iter()
        .skip(start as usize)
        .take(limit.unwrap_or(0) as usize)
        .map(|token_id| self.sales.get(&format!("{}{}{}", contract_id, ".", token_id)).unwrap())
        .collect()
    }

}