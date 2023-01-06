#![no_std]

use gmeta::metawasm;

#[metawasm]
pub trait Metawasm {
    type State = <NFTMetadata as Metadata>::State;
}
