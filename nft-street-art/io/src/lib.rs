#![no_std]

use gear_lib::non_fungible_token::{
    io::{NFTApproval, NFTTransfer, NFTTransferPayout},
    royalties::*,
    token::*,
};
use gstd::{prelude::*, ActorId};

pub use gear_lib::non_fungible_token::delegated::DelegatedApproveMessage;
use primitive_types::H256;

pub type Country = String;
pub type City = String;

#[derive(Debug, Encode, Decode, TypeInfo, Default, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Metadata {
    pub name: String,
    pub collection_name: Option<String>,
    pub description: Option<String>,
    pub social_network: String,
    pub country_and_city: Option<(Country, City)>,
    pub link_to_media: Option<String>,
    // link to json with coordinates
    pub coordinates: Option<String>,
    pub created_at: Option<u64>,
    pub child_token_id: Option<TokenId>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
#[allow(clippy::too_many_arguments)]
pub enum NFTAction {
    Mint {
        transaction_id: u64,
        account: ActorId,
        name: String,
        collection_name: String,
        description: String,
        social_network: String,
        country_and_city: Option<(Country, City)>,
        link_to_media: Option<String>,
        coordinates: Option<String>,
    },
    GenerateToken {
        parent_id: TokenId,
        owner: ActorId,
    },
    UpdateToken {
        token_id: TokenId,
        link_to_media: Option<String>,
        coordinates: Option<String>,
        description: Option<String>,
        collection_name: Option<String>,
    },
    Burn {
        transaction_id: u64,
        token_id: TokenId,
    },
    AddCountriesAndCities {
        countries_and_cities: Vec<(Country, Vec<City>)>,
    },
    RemoveCountriesAndCities {
        countries_and_cities: Vec<(Country, Option<Vec<City>>)>,
    },
    Transfer {
        transaction_id: u64,
        to: ActorId,
        token_id: TokenId,
    },
    TransferPayout {
        transaction_id: u64,
        to: ActorId,
        token_id: TokenId,
        amount: u128,
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
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitNFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
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
    TransactionMade,
    TokenUpdated,
    CountryCitiesAdded,
    CountryCitiesRemoved,
}

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum MyNFTQuery {
    Token { token_id: TokenId },
    TokensForOwner { owner: ActorId },
    TotalSupply,
    SupplyForOwner { owner: ActorId },
}

#[derive(Encode, Decode, TypeInfo, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
#[allow(clippy::large_enum_variant)]
pub enum MyNFTQueryReply {
    Token { metadata: Metadata },
    TokensForOwner { tokens: Vec<TokenId> },
    TotalSupply { total_supply: u128 },
    SupplyForOwner { supply: u128 },
}
