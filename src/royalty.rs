use crate::*;

pub trait NftRoyalty {
    fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout;

    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[near_bindgen]
impl NftRoyalty for NftContract {
    fn nft_payout(&self, token_id: TokenId, balance: U128, max_len_payout: u32) -> Payout {
        let token = self
            .tokens_by_id
            .get(&token_id)
            .expect("Token doesn't exits");
        let owner_id = token.owner_id;
        let royalty = token.royalty;
        let balance_u128 = u128::from(balance);
        let mut total_perpetual = 0;
        let mut payout_object = Payout {
            payout: HashMap::new(),
        };

        assert!(
            max_len_payout >= royalty.len() as u32,
            "Market can not payout to that many receivers"
        );

        for (k, v) in royalty.iter() {
            let key = k.clone();

            if key != owner_id {
                payout_object
                    .payout
                    .insert(key, royalty_to_payout(*v, balance_u128));
                total_perpetual += *v;
            }
        }

        payout_object.payout.insert(
            owner_id,
            royalty_to_payout(10_000 - total_perpetual, balance_u128),
        );

        payout_object
    }

    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        assert_one_yocto();

        let sender_id = env::predecessor_account_id();

        let previous_token =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, Some(approval_id), memo);

        refund_approved_account_ids(
            previous_token.owner_id.clone(),
            &previous_token.approved_account_ids,
        );

        self.nft_payout(token_id, balance, max_len_payout)
    }
}
