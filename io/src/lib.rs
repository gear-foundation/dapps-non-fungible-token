#![no_std]

use core::cmp::Ordering;

use gear_lib::non_fungible_token::{
    io::{NFTApproval, NFTTransfer, NFTTransferPayout},
    royalties::*,
    state::NFTState,
    token::*,
};
use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub use gear_lib::non_fungible_token::delegated::DelegatedApproveMessage;
use primitive_types::H256;

pub struct NFTMetadata;

#[derive(Debug, Default, Encode, Decode, TypeInfo, Clone)]
pub struct Constraints {
    pub max_mint_count: Option<u32>,
    pub authorized_minters: BTreeSet<ActorId>,
    pub referrals: BTreeSet<Referral>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitNFT {
    pub collection: Collection,
    pub royalties: Option<Royalties>,
    pub constraints: Constraints,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct Collection {
    pub name: String,
    pub description: String,
}

impl Metadata for NFTMetadata {
    type Init = In<InitNFT>;
    type Handle = InOut<NFTAction, NFTEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = State;
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTAction {
    Mint {
        transaction_id: u64,
        token_metadata: TokenMetadata,
    },
    MintReferral {
        transaction_id: u64,
    },
    Burn {
        transaction_id: u64,
        token_id: TokenId,
    },
    NFTPayout {
        owner: ActorId,
        amount: u128,
    },
    Approve {
        transaction_id: u64,
        to: ActorId,
        token_id: TokenId,
    },
    DelegatedApprove {
        transaction_id: u64,
        message: DelegatedApproveMessage,
        signature: [u8; 64],
    },
    Owner {
        token_id: TokenId,
    },
    IsApproved {
        to: ActorId,
        token_id: TokenId,
    },
    Clear {
        transaction_hash: H256,
    },
    AddMinter {
        transaction_id: u64,
        minter_id: ActorId,
    },
    AddReferral {
        transaction_id: u64,
        referral_id: ActorId,
    },
    AddReferralMetadata(TokenMetadata),
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone)]
pub enum NFTEvent {
    Transfer(NFTTransfer),
    TransferPayout(NFTTransferPayout),
    NFTPayout(Payout),
    Approval(NFTApproval),
    Owner {
        owner: ActorId,
        token_id: TokenId,
    },
    IsApproved {
        to: ActorId,
        token_id: TokenId,
        approved: bool,
    },
    MinterAdded {
        minter_id: ActorId,
    },
    ReferralAdded {
        referral_id: ActorId,
    },
}

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
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

#[derive(Debug, Clone, Default, Encode, Decode, TypeInfo)]
pub struct IoNFT {
    pub token: IoNFTState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub transactions: Vec<(H256, NFTEvent)>,
}

impl From<&NFTState> for IoNFTState {
    fn from(value: &NFTState) -> Self {
        let NFTState {
            name,
            symbol,
            base_uri,
            owner_by_id,
            token_approvals,
            token_metadata_by_id,
            tokens_for_owner,
            royalties,
        } = value;

        let owner_by_id = owner_by_id
            .iter()
            .map(|(hash, actor_id)| (*hash, *actor_id))
            .collect();

        let token_approvals = token_approvals
            .iter()
            .map(|(key, approvals)| (*key, approvals.iter().copied().collect()))
            .collect();

        let token_metadata_by_id = token_metadata_by_id
            .iter()
            .map(|(id, metadata)| (*id, metadata.clone()))
            .collect();

        let tokens_for_owner = tokens_for_owner
            .iter()
            .map(|(id, tokens)| (*id, tokens.clone()))
            .collect();

        Self {
            name: name.clone(),
            symbol: symbol.clone(),
            base_uri: base_uri.clone(),
            owner_by_id,
            token_approvals,
            token_metadata_by_id,
            tokens_for_owner,
            royalties: royalties.clone(),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct Nft {
    pub owner: ActorId,
    pub name: String,
    pub description: String,
    pub media_url: String,
    pub attrib_url: String,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct State {
    pub tokens: Vec<(TokenId, Nft)>,
    pub owner: ActorId,
    pub transactions: Vec<(H256, NFTEvent)>,
    pub owners: Vec<(ActorId, TokenId)>,
    pub collection: Collection,
    pub nonce: TokenId,
    pub constraints: Constraints,
}

#[derive(Debug, Encode, Decode, TypeInfo, Clone, Copy)]
pub struct Referral {
    pub id: ActorId,
}

impl PartialEq for Referral {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Referral {}

impl PartialOrd for Referral {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Referral {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
