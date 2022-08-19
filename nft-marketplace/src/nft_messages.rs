use gstd::{msg, prelude::*, ActorId};
pub type Payout = BTreeMap<ActorId, u128>;
use crate::{ContractId, TokenId};
use gear_lib::non_fungible_token::io::*;
use market_io::DelegatedApprove;
use nft_io::*;

pub async fn nft_transfer(
    nft_program_id: ContractId,
    to: ActorId,
    token_id: TokenId,
    amount: u128,
) -> Payout {
    let response: Vec<u8> = msg::send_for_reply_as(
        nft_program_id,
        NFTAction::TransferPayout {
            to,
            token_id,
            amount,
        },
        0,
    )
    .expect("can't send message")
    .await
    .expect("error in transfer");
    let decoded_response: NFTTransferPayout =
        NFTTransferPayout::decode(&mut &response[..]).expect("Error in decoding payouts");
    decoded_response.payouts
}

pub async fn nft_approve(delegated_approve: &DelegatedApprove) {
    msg::send_for_reply(
        delegated_approve.message.nft_program_id,
        NFTAction::DelegatedApprove {
            message: delegated_approve.message.clone(),
            signature: delegated_approve.signature,
        },
        0,
    )
    .expect("can't send message")
    .await
    .expect("error in transfer");
}
