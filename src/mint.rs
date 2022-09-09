use crate::*;

#[near_bindgen]
impl NftContract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) {
        assert!(
            env::attached_deposit() > 0,
            "Deposit needs to be greater than 0"
        );
        let initial_storage_usage = env::storage_usage();

        let mut royalty = HashMap::default();

        if let Some(perpetual_royalties) = perpetual_royalties {
            // make sure GAS enough to pay out
            assert!(
                perpetual_royalties.len() <= 6,
                "Cannot add more than 6 perpetual royalty amounts"
            );

            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        let token = Token {
            owner_id: receiver_id,
            approved_account_ids: HashMap::default(),
            next_approval_id: 0,
            royalty,
        };

        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token with id already exists"
        );
        self.token_metadata_by_id.insert(&token_id, &metadata);

        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        let nft_mint_log = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: token.owner_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo: None,
            }]),
        };

        env::log_str(&nft_mint_log.to_string());

        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        refund_deposit(required_storage_in_bytes);
    }
}
