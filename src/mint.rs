use crate::*;

#[near_bindgen]
impl NftContract {
    #[payable]
    pub fn nft_mint(&mut self, token_id: TokenId, metadata: TokenMetadata, receiver_id: AccountId) {
        assert!(
            env::attached_deposit() > 0,
            "Deposit needs to be greater than 0"
        );
        let initial_storage_usage = env::storage_usage();

        let token = Token {
            owner_id: receiver_id,
            approved_account_ids: HashMap::default(),
            next_approval_id: 0,
        };

        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token with id already exists"
        );
        self.token_metadata_by_id.insert(&token_id, &metadata);

        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        refund_deposit(required_storage_in_bytes);
    }
}
