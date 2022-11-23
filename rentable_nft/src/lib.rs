#![no_std]

use nft::NFT;
use gear_lib::non_fungible_token::{nft_core::NFTCore, token::TokenId, state::{NFTQueryReply, NFTQuery}};
use gstd::{exec::block_timestamp, msg, ActorId, BTreeMap, ToString};
use io::{NFTAction, NFTEvent, InitNFT};

#[derive(Debug, Default)]
pub struct UserInfo {
    pub address: ActorId, // address of user role
    pub expires: u64,     // unix timestamp
}

#[derive(Debug, Default)]
pub struct RentableNFT {
    pub nft: NFT,
    pub users_info: BTreeMap<TokenId, UserInfo>,
}

static mut CONTRACT: Option<RentableNFT> = None;

#[no_mangle]
unsafe extern "C" fn handle() {
    let action: NFTAction = msg::load().expect("Could not load NFTAction");
    let rentable_nft = CONTRACT.get_or_insert(Default::default());
    match action {
        NFTAction::SetUser {
            token_id,
            address,
            expires,
        } => {
            rentable_nft.set_user(address, token_id, expires);
            let payload = NFTEvent::UpdateUser {
                token_id,
                address,
                expires,
            };
            msg::reply(payload, 0).expect("Error during replying with `NFTEvent::SetUser`");
        }
        NFTAction::UserOf { token_id } => {
            let address = rentable_nft.user_of(&token_id);
            let payload = NFTEvent::UserOf { address };
            msg::reply(payload, 0).expect("Error during replying with `NFTEvent::UserOf`");
        }
        NFTAction::UserExpires { token_id } => {
            let expires = rentable_nft.user_expires(&token_id);
            let payload = NFTEvent::UserExpires { expires };
            msg::reply(payload, 0).expect("Error during replying with `NFTEvent::UserExpires`");
        }
    };
}

gstd::metadata! {
    title: "RentableNFT",
    init:
        input: InitNFT,
    handle:
        input: NFTAction,
        output: NFTEvent,
    state:
        input: NFTQuery,
        output: NFTQueryReply,
}

impl RentableNFT {
    fn set_user(&mut self, address: ActorId, token_id: TokenId, expires: u64) {
        self.nft.assert_zero_address(&address);

        let owner = &self.nft.owner;

        // is Approved or Owner
        if !self.nft.is_approved_to(&msg::source(), token_id) {
            self.nft.assert_owner(owner);
        }

        self.users_info
            .entry(token_id)
            .and_modify(|user_info| user_info.expires = expires)
            .or_insert(UserInfo { address, expires });
    }

    fn user_of(&self, token_id: &TokenId) -> ActorId {
        if let Some(user_info) = self.users_info.get(token_id) {
            if user_info.expires < block_timestamp() {
                return user_info.address;
            }
        }

        ActorId::zero()
    }

    fn user_expires(&self, token_id: &TokenId) -> u64 {
        if let Some(user_info) = self.users_info.get(token_id) {
            user_info.expires
        } else {
            0u64
        }
    }
}
