use crate::*;

pub trait NftApproval {
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);

    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool;

    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);

    fn nft_revoke_all_accounts(&mut self, token_id: TokenId);
}

#[near_bindgen]
impl NftApproval for NftContract {
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) {
        assert_at_least_one_yocto();

        let mut token = self
            .tokens_by_id
            .get(&token_id)
            .expect("Token doesn't exist.");

        assert_eq!(
            env::predecessor_account_id(),
            token.owner_id,
            "Predecessor should be the owner of NFT"
        );

        let approval_id = token.next_approval_id;

        let is_new_approval = token
            .approved_account_ids
            .insert(account_id.clone(), approval_id)
            .is_none();

        let storage_used = if is_new_approval {
            bytes_for_approved_account_id(&account_id)
        } else {
            0
        };

        token.next_approval_id += 1;
        self.tokens_by_id.insert(&token_id, &token);

        refund_deposit(storage_used);

        if let Some(msg) = msg {
            ext_nft_approval_receiver::ext(account_id).nft_on_approve(
                token_id,
                token.owner_id,
                approval_id,
                msg,
            );
        }
    }

    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        let token = self
            .tokens_by_id
            .get(&token_id)
            .expect("Token doesn't exist");

        if let Some(actual_approval_id) = token.approved_account_ids.get(&approved_account_id) {
            if let Some(given_approval_id) = approval_id {
                *actual_approval_id == given_approval_id
            } else {
                true
            }
        } else {
            false
        }
    }

    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        assert_one_yocto();

        let mut token = self
            .tokens_by_id
            .get(&token_id)
            .expect("Token doesn't exists");

        let predecessor_id = env::predecessor_account_id();

        assert_eq!(predecessor_id, token.owner_id.clone());

        if token.approved_account_ids.remove(&account_id).is_some() {
            self.tokens_by_id.insert(&token_id, &token);
            refund_approved_account_ids_iter(predecessor_id, [account_id].iter());
        }
    }

    #[payable]
    fn nft_revoke_all_accounts(&mut self, token_id: TokenId) {
        assert_one_yocto();

        let mut token = self
            .tokens_by_id
            .get(&token_id)
            .expect("Token doesn't exists");

        let predecessor_id = env::predecessor_account_id();

        assert_eq!(predecessor_id, token.owner_id.clone());

        if !token.approved_account_ids.is_empty() {
            let approved_account_ids = &token.approved_account_ids.clone();

            token.approved_account_ids.clear();
            self.tokens_by_id.insert(&token_id, &token);

            refund_approved_account_ids(predecessor_id, approved_account_ids);
        }
    }
}

#[ext_contract(ext_nft_approval_receiver)]
trait NftApprovalReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}
