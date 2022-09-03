use crate::*;

pub trait NftCore {
    fn nft_json(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[near_bindgen]
impl NftCore for NftContract {
    fn nft_json(&self, token_id: TokenId) -> Option<JsonToken> {
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
