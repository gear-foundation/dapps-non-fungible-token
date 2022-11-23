#![no_std]

pub use gear_lib::non_fungible_token::delegated::DelegatedApproveMessage;
use gear_lib::non_fungible_token::{
    royalties::*,
    token::*,
};
use gstd::{prelude::*, ActorId};

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum NFTAction {
    SetUser {
        token_id: TokenId,
        address: ActorId,
        expires: u64, // unix timestamp
    },
    UserOf {
        token_id: TokenId,
    },
    UserExpires {
        token_id: TokenId,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitNFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub royalties: Option<Royalties>,
}

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum NFTEvent {
    UpdateUser {
        token_id: TokenId,
        address: ActorId,
        expires: u64,
    },
    UserOf {
        address: ActorId,
    },
    UserExpires {
        expires: u64,
    },
}
