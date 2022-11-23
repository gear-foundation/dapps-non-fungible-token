#![no_std]

use gear_lib::non_fungible_token::{nft_core::*, state::*};
use gstd::{msg, prelude::*};
use nft_io::{InitNFT, NFTAction, NFTEvent};

use nft_common::{MyNFTCore, NFT};

static mut CONTRACT: Option<NFT> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitNFT = msg::load().expect("Unable to decode InitNFT");
    if config.royalties.is_some() {
        config.royalties.as_ref().unwrap().validate();
    }
    let nft = NFT {
        token: NFTState {
            name: config.name,
            symbol: config.symbol,
            base_uri: config.base_uri,
            royalties: config.royalties,
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
            token_metadata,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(NFTEvent::Transfer(MyNFTCore::mint(nft, token_metadata)), 0)
                    .expect("Error during replying with `NFTEvent::Transfer`");
            }
        }
        NFTAction::Burn {
            transaction_id,
            token_id,
        } => {
            if !nft.transaction_made(transaction_id) {
                msg::reply(NFTEvent::Transfer(NFTCore::burn(nft, token_id)), 0)
                    .expect("Error during replying with `NFTEvent::Transfer`");
            }
        }
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
    let query: NFTQuery = msg::load().expect("failed to decode input argument");
    let nft = CONTRACT.get_or_insert(NFT::default());
    let encoded =
        NFTMetaState::proc_state(nft, query).expect("Error in reading NFT contract state");
    gstd::util::to_leak_ptr(encoded)
}

gstd::metadata! {
    title: "NFT",
    init:
        input: InitNFT,
    handle:
        input: NFTAction,
        output: NFTEvent,
    state:
        input: NFTQuery,
        output: NFTQueryReply,
}
