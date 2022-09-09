use crate::*;

const GAS_FOR_NFT_ON_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(25_000_000_000_000);

pub trait NftCore {
    // transfer an NFT to a receiver ID
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );

    // transfer an NFT to a reveicerId and call a function on receiver's contract
    /// return `true` if the token was transferred from the sender's account
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool>;

    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[ext_contract(ext_nft_receiver)]
trait NftReceiver {
    /// Return `true` if the token should be returned back to the sender.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NftResolver {
    /// Return `true` if receiver successfully received NFT and `false` if NFT should be returned to original owner
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,
    ) -> bool;
}

#[near_bindgen]
impl NftCore for NftContract {
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let previous_token =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);

        refund_approved_account_ids(
            previous_token.owner_id,
            &previous_token.approved_account_ids,
        );
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let previous_token = self.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            approval_id,
            memo.clone(),
        );

        let mut authorized_id = None;
        if sender_id != previous_token.owner_id {
            authorized_id = Some(sender_id.to_string());
        }

        ext_nft_receiver::ext(receiver_id.clone())
            .with_static_gas(GAS_FOR_NFT_ON_TRANSFER)
            .nft_on_transfer(
                sender_id,
                previous_token.owner_id.clone(),
                token_id.clone(),
                msg,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .nft_resolve_transfer(
                        authorized_id,
                        previous_token.owner_id,
                        receiver_id,
                        token_id,
                        previous_token.approved_account_ids,
                        memo,
                    ),
            )
            .into()
    }

    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        let token = self.tokens_by_id.get(&token_id);
        let metadata = self.token_metadata_by_id.get(&token_id).unwrap();

        match token {
            Some(t) => Some(JsonToken {
                token_id,
                owner_id: t.owner_id,
                metadata,
                approved_account_ids: t.approved_account_ids,
                royalty: t.royalty,
            }),
            None => None,
        }
    }
}

#[near_bindgen]
impl NftResolver for NftContract {
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: HashMap<AccountId, u64>,
        memo: Option<String>,
    ) -> bool {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Promise returns too many results"
        );
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            if let Ok(return_token) = serde_json::from_slice::<bool>(&value) {
                if !return_token {
                    refund_approved_account_ids(owner_id, &approved_account_ids);
                    return true;
                }
            }
        }

        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            if token.owner_id != receiver_id {
                // This case probably doesn't trigger, because if internal_transfer is successful, receiver_id will own this NFT,
                // otherwise the code will panic.
                refund_approved_account_ids(owner_id, &approved_account_ids);
                return true;
            }
            token
        } else {
            // This case doesn't trigger because internal_transfer function has already checked token_id
            // refund_approved_account_ids(owner_id, &approved_account_ids);
            return true;
        };

        self.internal_remove_token_from_owner(&receiver_id, &token_id);
        self.internal_add_token_to_owner(&owner_id, &token_id);

        token.owner_id = owner_id.clone();
        token.approved_account_ids = approved_account_ids;
        self.tokens_by_id.insert(&token_id, &token);

        let nft_transfer_log = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                authorized_id,
                old_owner_id: receiver_id.to_string(),
                new_owner_id: owner_id.to_string(),
                token_ids: vec![token_id],
                memo,
            }]),
        };

        env::log_str(&nft_transfer_log.to_string());

        false
    }
}
