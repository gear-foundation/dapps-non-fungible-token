#![no_std]

use gmeta::{metawasm, Metadata};
use gstd::ToString;
use nft_io::NFTMetadata;

#[metawasm]
pub trait Metawasm {
    type State = <NFTMetadata as Metadata>::State;

    fn foo(state: Self::State) -> u64 {
        nft_io::foo(state)
    }
}
