use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, serde_json, AccountId, Balance, BorshStorageKey, CryptoHash,
    Gas, PanicOnDefault, Promise, PromiseOrValue, PromiseResult,
};
mod enumeration;
mod internal;
mod metadata;
mod mint;
mod nft_core;

pub use crate::enumeration::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NftContract {
    pub owner_id: AccountId,

    pub metadata: LazyOption<NFTContractMetadata>,

    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    pub tokens_by_id: LookupMap<TokenId, Token>,

    pub token_metadata_by_id: LookupMap<TokenId, TokenMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
}

#[near_bindgen]
impl NftContract {
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        Self {
            owner_id,
            metadata: LazyOption::new(StorageKey::NFTContractMetadata, Some(&metadata)),
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner),
            tokens_by_id: LookupMap::new(StorageKey::TokensById),
            token_metadata_by_id: LookupMap::new(StorageKey::TokenMetadataById),
        }
    }

    #[init]
    pub fn new_default_metadata(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "NFT For Learning".to_string(),
                symbol: "NFL".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, ONE_NEAR};

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .account_balance(10)
            .current_account_id(alice())
            .signer_account_id(alice())
            .predecessor_account_id(alice())
            .is_view(is_view);
        builder
    }

    fn init_nft_contract() -> NftContract {
        NftContract::new_default_metadata(alice())
    }

    fn mint_nft(contract: &mut NftContract, account_id: AccountId, token_id: TokenId) {
        let token_metadata = TokenMetadata {
            title: None,
            description: None,
            media: None,
            media_hash: None,
            copies: None,
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        };

        contract.nft_mint(token_id, token_metadata, account_id);
    }

    #[test]
    fn test_initialize_metadata() {
        let mut context = get_context(false);
        context.attached_deposit(ONE_NEAR);

        testing_env!(context.build());

        let contract = init_nft_contract();

        let nft_contract_metadata = contract.nft_metadata();
        assert_eq!(nft_contract_metadata.spec, "nft-1.0.0".to_string());
        assert_eq!(nft_contract_metadata.name, "NFT For Learning".to_string());
        assert_eq!(nft_contract_metadata.symbol, "NFL".to_string());
    }

    #[test]
    fn test_mint_nft() {
        let mut context = get_context(false);
        context.attached_deposit(ONE_NEAR);

        testing_env!(context.build());

        let mut contract = init_nft_contract();

        mint_nft(&mut contract, alice(), "token#1".to_string());
        let json_token = contract.nft_token("token#1".to_string());
        assert!(json_token.is_some());
        assert_eq!(json_token.unwrap().owner_id, alice());
    }
}
