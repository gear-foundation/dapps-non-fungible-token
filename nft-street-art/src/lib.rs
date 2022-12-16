#![no_std]

use gear_lib::non_fungible_token::{io::NFTTransfer, nft_core::*, state::*, token::*};
use gear_lib_derive::{NFTCore, NFTStateKeeper};
use gstd::{msg, prelude::*, ActorId};
use nft_art_io::*;
use primitive_types::{H256, U256};

pub mod implementation;
pub use implementation::MyNFTCore;

#[derive(Debug, Default, NFTStateKeeper, NFTCore)]
pub struct NFT {
    #[NFTStateField]
    pub token: NFTState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub countries_to_cities: BTreeMap<Country, BTreeSet<City>>,
    pub id_to_metadata: BTreeMap<TokenId, Metadata>,
    pub transactions: BTreeSet<H256>,
}

static mut CONTRACT: Option<NFT> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitNFT = msg::load().expect("Unable to decode InitNFT");

    let nft = NFT {
        token: NFTState {
            name: config.name,
            symbol: config.symbol,
            base_uri: config.base_uri,
            ..Default::default()
        },
        owner: msg::source(),
        ..Default::default()
    };
    CONTRACT = Some(nft);
}

#[no_mangle]
unsafe extern "C" fn handle() {
    let action: NFTAction = msg::load().expect("Could not load NFTAction");
    let nft = CONTRACT.get_or_insert(Default::default());
    match action {
        NFTAction::Mint {
            transaction_id,
            account,
            name,
            collection_name,
            description,
            social_network,
            country_and_city,
            link_to_media,
            coordinates,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(
                    NFTEvent::Transfer(MyNFTCore::mint(
                        nft,
                        &account,
                        name,
                        collection_name,
                        description,
                        social_network,
                        country_and_city,
                        link_to_media,
                        coordinates,
                    )),
                    0,
                )
                .expect("Error during replying with `NFTEvent::Transfer`");
            }
        }
        NFTAction::GenerateToken { parent_id, owner } => {
            msg::reply(
                NFTEvent::Transfer(MyNFTCore::generate_token(nft, &owner, parent_id)),
                0,
            )
            .expect("Error during replying with `NFTEvent::Transfer`");
        }
        NFTAction::UpdateToken {
            token_id,
            link_to_media,
            coordinates,
            description,
            collection_name,
        } => nft.update_token(
            token_id,
            link_to_media,
            coordinates,
            description,
            collection_name,
        ),
        NFTAction::Burn {
            transaction_id,
            token_id,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(NFTEvent::Transfer(NFTCore::burn(nft, token_id)), 0)
                    .expect("Error during replying with `NFTEvent::Transfer`");
            }
        }
        NFTAction::AddCountriesAndCities {
            countries_and_cities,
        } => nft.add_countries_and_cities(&countries_and_cities),
        NFTAction::RemoveCountriesAndCities {
            countries_and_cities,
        } => nft.remove_countries_and_cities(&countries_and_cities),
        NFTAction::Transfer {
            transaction_id,
            to,
            token_id,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(NFTEvent::Transfer(NFTCore::transfer(nft, &to, token_id)), 0)
                    .expect("Error during replying with `NFTEvent::Transfer`");
            }
        }
        NFTAction::TransferPayout {
            transaction_id,
            to,
            token_id,
            amount,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(
                    NFTEvent::TransferPayout(NFTCore::transfer_payout(nft, &to, token_id, amount)),
                    0,
                )
                .expect("Error during replying with `NFTEvent::TransferPayout`");
            }
        }
        NFTAction::NFTPayout { owner, amount } => {
            msg::reply(
                NFTEvent::NFTPayout(NFTCore::nft_payout(nft, &owner, amount)),
                0,
            )
            .expect("Error during replying with `NFTEvent::NFTPayout`");
        }
        NFTAction::Approve {
            transaction_id,
            to,
            token_id,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(NFTEvent::Approval(NFTCore::approve(nft, &to, token_id)), 0)
                    .expect("Error during replying with `NFTEvent::Approval`");
            }
        }
        NFTAction::Owner { token_id } => {
            msg::reply(
                NFTEvent::Owner {
                    owner: NFTCore::owner_of(nft, token_id),
                    token_id,
                },
                0,
            )
            .expect("Error during replying with `NFTEvent::Owner`");
        }
        NFTAction::IsApproved { to, token_id } => {
            msg::reply(
                NFTEvent::IsApproved {
                    to,
                    token_id,
                    approved: NFTCore::is_approved_to(nft, &to, token_id),
                },
                0,
            )
            .expect("Error during replying with `NFTEvent::IsApproved`");
        }
        NFTAction::DelegatedApprove {
            transaction_id,
            message,
            signature,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(
                    NFTEvent::Approval(NFTCore::delegated_approve(nft, message, signature)),
                    0,
                )
                .expect("Error during replying with `NFTEvent::Approval`");
            }
        }
        NFTAction::Clear { transaction_hash } => nft.clear(transaction_hash),
    };
}

#[no_mangle]
unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: MyNFTQuery = msg::load().expect("failed to decode input argument");
    let nft = CONTRACT.get_or_insert(NFT::default());
    let encoded = match query {
        MyNFTQuery::Token { token_id } => {
            let metadata = nft
                .id_to_metadata
                .get(&token_id)
                .unwrap_or(&Default::default())
                .clone();
            MyNFTQueryReply::Token { metadata }
        }
        MyNFTQuery::TokensForOwner { owner } => {
            let mut tokens: Vec<TokenId> = Vec::new();
            if let Some(token_ids) = nft.get().tokens_for_owner.get(&owner) {
                for token_id in token_ids {
                    tokens.push(*token_id);
                }
            }
            MyNFTQueryReply::TokensForOwner { tokens }
        }
        MyNFTQuery::TotalSupply => {
            let total_supply = nft.get().owner_by_id.len() as u128;
            MyNFTQueryReply::TotalSupply { total_supply }
        }
        MyNFTQuery::SupplyForOwner { owner } => {
            let supply = nft
                .get()
                .tokens_for_owner
                .get(&owner)
                .map(|tokens| tokens.len() as u128)
                .unwrap_or(0);
            MyNFTQueryReply::SupplyForOwner { supply }
        }
    }
    .encode();
    gstd::util::to_leak_ptr(encoded)
}
