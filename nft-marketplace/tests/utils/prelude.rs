pub use super::{Market, Program};
pub use gstd::prelude::*;
use gstd::ActorId;
use hex_literal::hex;
pub use nft_marketplace::*;
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};

pub const BUYER: u64 = 100;
pub const NFT_PRICE: u128 = 100_000;
const SELLER_SEED: [u8; 32] =
    hex!("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
pub const ADMIN: u64 = 200;
pub const TREASURY_ID: u64 = 300;
pub const TREASURY_FEE: u16 = 3;
pub const TOKEN_ID: u64 = 0;
pub const BID_PERIOD: u64 = 3_600_000;
pub const DURATION: u64 = 86_400_000;
pub const PARTICIPANTS: &[u64] = &[500, 501, 502, 503, 504];
pub const MARKET_ID: u64 = 3;
pub const MIN_BID_PERIOD: u64 = 60_000;

pub fn seller_pair() -> Sr25519Pair {
    Sr25519Pair::from_seed(&SELLER_SEED)
}

pub fn seller_actor_id() -> ActorId {
    seller_pair().public().0.into()
}
