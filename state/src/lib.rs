#![no_std]

use gear_lib::non_fungible_token::state::NFTQueryReply;
use gmeta::{metawasm, Metadata};
use gstd::String;
use nft_io::NFTMetadata;

#[metawasm]
pub mod metafns {
    pub type State = <NFTMetadata as Metadata>::State;

    pub fn info(state: State) -> NFTQueryReply {
        NFTQueryReply::NFTInfo {
            name: state.collection.name,
            symbol: String::new(),
            base_uri: String::new(),
        }
    }
}
