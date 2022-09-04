use crate::*;

const GAS_FOR_NFT_ON_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(25_000_000_000_000);

pub trait NftCore {
    // transfer an NFT to a receiver ID
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>);

    // transfer an NFT to a reveicerId and call a function on receiver's contract
    /// return `true` if the token was transferred from the sender's account
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
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
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
    ) -> bool;
}

#[near_bindgen]
impl NftCore for NftContract {
    #[payable]
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.internal_transfer(&sender_id, &receiver_id, &token_id, memo);
    }

    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();

        let transferred_token = self.internal_transfer(&sender_id, &receiver_id, &token_id, memo);

        ext_nft_receiver::ext(receiver_id.clone())
            .with_static_gas(GAS_FOR_NFT_ON_TRANSFER)
            .nft_on_transfer(
                sender_id,
                transferred_token.owner_id.clone(),
                token_id.clone(),
                msg,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .nft_resolve_transfer(transferred_token.owner_id, receiver_id, token_id),
            )
            .into()
    }

    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        let token = self.tokens_by_id.get(&token_id);
        match token {
            Some(t) => Some(JsonToken {
                token_id: token_id.clone(),
                owner_id: t.owner_id,
                metadata: self.token_metadata_by_id.get(&token_id).unwrap(),
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
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
    ) -> bool {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Promise returns too many results"
        );
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            if let Ok(return_token) = serde_json::from_slice::<bool>(&value) {
                if !return_token {
                    return true;
                }
            }
        }

        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            if token.owner_id != receiver_id {
                return true;
            }
            token
        } else {
            return true;
        };

        self.internal_remove_token_from_owner(&receiver_id, &token_id);
        self.internal_add_token_to_owner(&owner_id, &token_id);

        token.owner_id = owner_id;
        self.tokens_by_id.insert(&token_id, &token);

        false
    }
}
