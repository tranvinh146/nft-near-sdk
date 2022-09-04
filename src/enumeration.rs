use crate::*;

#[near_bindgen]
impl NftContract {
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonToken> {
        let tokens_set_for_owner = self.tokens_per_owner.get(&account_id);
        let tokens = if let Some(tokens_set) = tokens_set_for_owner {
            tokens_set
        } else {
            return vec![];
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));
        let limit = u128::from(limit.unwrap_or(0));

        tokens
            .iter()
            .skip(start as usize)
            .take(limit as usize)
            .map(|token_id| self.nft_token(token_id).unwrap())
            .collect()
    }
}
