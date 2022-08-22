use gstd::{msg, prelude::*, ActorId};
pub type Payout = BTreeMap<ActorId, u128>;
use crate::{ContractId, TokenId};
use gear_lib::non_fungible_token::io::*;
use nft_io::*;

pub async fn nft_transfer(
    nft_program_id: ContractId,
    to: ActorId,
    token_id: TokenId,
    amount: u128,
) -> Payout {
    let response: NFTEvent = msg::send_for_reply_as(
        nft_program_id,
        NFTAction::TransferPayout {
            to,
            token_id,
            amount,
        },
        0,
    )
    .unwrap()
    .await
    .expect("error in transfer");
    if let NFTEvent::TransferPayout {payouts, ..} = response {
        return payouts
    } else {
        panic!("Wrong received answer");
    }
}

pub async fn nft_approve(nft_program_id: ContractId, to: ActorId, token_id: TokenId) {
    let _approve_response: NFTEvent =
        msg::send_for_reply_as(nft_program_id, NFTAction::Approve { to, token_id }, 0)
            .unwrap()
            .await
            .expect("error in approve");
}
