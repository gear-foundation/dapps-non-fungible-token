#![no_std]

use gear_lib::non_fungible_token::{royalties::*, state::*, token::*};
use gstd::{prelude::*, ActorId};

pub type LayerId = u128;
pub type ItemId = u128;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum OnChainNFTQuery {
    TokenURI { token_id: TokenId },
    Base(NFTQuery),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum OnChainNFTAction {
    Mint {
        token_metadata: TokenMetadata,
        description: Vec<ItemId>,
    },
    Burn {
        token_id: TokenId,
    },
    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
    Approve {
        to: ActorId,
        token_id: TokenId,
    },
    TransferPayout {
        to: ActorId,
        token_id: TokenId,
        amount: u128,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct TokenURI {
    pub metadata: TokenMetadata,
    pub content: Vec<String>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitOnChainNFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub base_image: String,
    pub layers: BTreeMap<LayerId, Vec<String>>,
    pub royalties: Option<Royalties>,
}
