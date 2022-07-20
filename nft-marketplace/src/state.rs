use crate::{Item, TokenId, ContractId};
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    AllItems,
    ItemInfo {
        nft_contract_id: ContractId,
        token_id: TokenId,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum StateReply {
    AllItems(Vec<Item>),
    ItemInfo(Item),
}
