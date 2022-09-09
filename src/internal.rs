use crate::*;

impl NftContract {
    pub(crate) fn internal_add_token_to_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(StorageKey::TokenPerOwnerInner {
                account_id_hash: hash_account_id(&account_id),
            })
        });
        tokens_set.insert(token_id);
        self.tokens_per_owner.insert(account_id, &tokens_set);
    }

    pub(crate) fn internal_remove_token_from_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        let mut tokens_set = self
            .tokens_per_owner
            .get(account_id)
            .expect(&format!("{} doesn't own {} NFT", account_id, token_id));

        tokens_set.remove(token_id);

        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        } else {
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Token {
        let token = self
            .tokens_by_id
            .get(token_id)
            .expect("Token doesn't exist");

        if sender_id != &token.owner_id {
            if let Some(given_approval_id) = approval_id {
                let actual_approval_id = token
                    .approved_account_ids
                    .get(sender_id)
                    .expect("Sender is not approved");

                assert_eq!(
                    &given_approval_id, actual_approval_id,
                    "The given approval id is differrent from the actual approval id"
                );
            } else {
                env::panic_str("The approval id must not be empty")
            }
        }

        // assure receiver is not owner
        assert_ne!(
            &token.owner_id, receiver_id,
            "NFT owner and receiver should be different"
        );

        // remove NFT from sender
        self.internal_remove_token_from_owner(&token.owner_id, token_id);

        // add NFT to receiver
        self.internal_add_token_to_owner(receiver_id, token_id);

        // change owner of token
        let transferred_token = Token {
            owner_id: receiver_id.to_owned(),
            approved_account_ids: HashMap::default(),
            next_approval_id: token.next_approval_id,
            royalty: token.royalty.clone(),
        };
        self.tokens_by_id.insert(token_id, &transferred_token);

        if let Some(memo) = memo.as_ref() {
            env::log_str(&format!("Memo: {}", memo))
        }

        let mut authorized_id = None;

        if approval_id.is_some() {
            authorized_id = Some(sender_id.to_string())
        }

        let nft_transfer_log = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                authorized_id,
                old_owner_id: token.owner_id.to_string(),
                new_owner_id: receiver_id.to_string(),
                token_ids: vec![token_id.to_string()],
                memo,
            }]),
        };

        env::log_str(&nft_transfer_log.to_string());

        token
    }
}

pub(crate) fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);

    assert!(
        env::attached_deposit() >= required_cost,
        "Must attach {} yoctoNEAR to coverage storage",
        required_cost
    );

    let refund = env::attached_deposit() - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

pub(crate) fn refund_approved_account_ids_iter<'a, I>(
    account_id: AccountId,
    approved_account_ids: I,
) -> Promise
where
    I: Iterator<Item = &'a AccountId>,
{
    let storage_released: u64 = approved_account_ids
        .map(bytes_for_approved_account_id)
        .sum();
    Promise::new(account_id).transfer(Balance::from(storage_released) * env::storage_byte_cost())
}

pub(crate) fn refund_approved_account_ids(
    account_id: AccountId,
    approved_account_ids: &HashMap<AccountId, u64>,
) -> Promise {
    refund_approved_account_ids_iter(account_id, approved_account_ids.keys())
}

pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

pub(crate) fn assert_one_yocto() {
    assert_eq!(
        env::attached_deposit(),
        1,
        "Requiring attached deposit of EXACTLY 1 yoctoNear"
    );
}

pub(crate) fn assert_at_least_one_yocto() {
    assert!(
        env::attached_deposit() >= 1,
        "Requiring attached deposit of AT LEAST 1 yoctoNear"
    );
}

pub(crate) fn royalty_to_payout(royalty_percentage: u32, amount_to_pay: Balance) -> U128 {
    U128(royalty_percentage as u128 * amount_to_pay / 10_000)
}
