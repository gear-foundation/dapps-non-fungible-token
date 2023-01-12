#![no_std]

use gmeta::metawasm;

#[metawasm]
pub trait Metawasm {
    type State = <NFTMetadata as Metadata>::State;

    fn info(state: Self::State) -> NFTQueryReply::NFTInfo {
        NFTQueryReply(NFTInfo {
            name: state.name.clone(),
            symbol: state.symbol.clone(),
            base_uri: state:base_uri.clone()..
        })
    }

    fn token(token: U256, state: Self::State) -> NFTQueryReply::Token {
        let mut token = Token::default();
        if let Some(owner_id) = state.owner_by_id.get(&token_id) {
            token.id = token_id;
            token.owner_id = *owner_id;
        }
        if let Some(approved_account_ids) = state.token_approvals.get(&token_id) {
            token.approved_account_ids = approved_account_ids.iter().copied().collect();
        }
        if let Some(Some(metadata)) = state.token_metadata_by_id.get(&token_id) {
            token.name = metadata.name.clone();
            token.description = metadata.description.clone();
            token.media = metadata.media.clone();
            token.reference = metadata.reference.clone();
        }
        NFTQueryReply::Token {token}
    }

    fn tokens_for_owner(onwer: ActorId, state: Self::State) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        if let Some(token_ids) = state.tokens_for_owner.get(owner) {
            for token_id in token_ids {
                tokens.push(self.token(*token_id));
            }
        }
        tokens
    }
    fn total_supply(state: Self::State) -> u128 {
        state.owner_by_id.len() as u128

    }
    fn supply_for_owner(&self, owner: &ActorId) -> u128 {
        state
            .tokens_for_owner
            .get(owner)
            .map(|tokens| tokens.len() as u128)
            .unwrap_or(0)
    }
    fn all_tokens(state: Self::State) -> Vec<Token> {
        state
            .owner_by_id
            .keys()
            .map(|id| self.token(*id))
            .collect()
    }
    fn approved_tokens(state: Self::State, account: &ActorId) -> Vec<Token> {
        state
            .owner_by_id
            .keys()
            .filter_map(|id| {
                state.token_approvals.get(id).and_then(|approvals| {
                    if approvals.contains(account) {
                        Some(self.token(*id))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

pub struct IoNFTState {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub owner_by_id: Vec<(TokenId, ActorId)>,
    pub token_approvals: Vec<(TokenId, Vec<ActorId>)>,
    pub token_metadata_by_id: Vec<(TokenId, Option<TokenMetadata>)>,
    pub tokens_for_owner: Vec<(ActorId, Vec<TokenId>)>,
    pub royalties: Option<Royalties>,
}
