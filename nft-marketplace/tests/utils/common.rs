use super::prelude::*;
use super::{FungibleToken, Market, NonFungibleToken};
use core::fmt::Debug;
use gstd::ActorId;
use gtest::{Log, Program as InnerProgram, RunResult, System};

pub fn initialize_system() -> System {
    let system = System::new();
    system.init_logger();
    system
}

pub fn initialize_programs(system: &System) -> (FungibleToken, NonFungibleToken, Market) {
    let ft_program = FungibleToken::initialize(system);

    let nft_program = NonFungibleToken::initialize(system);
    nft_program.mint(SELLER);

    let market = Market::initialize(system);
    ft_program.approve(BUYER, market.actor_id(), NFT_PRICE);

    market
        .add_ft_contract(ADMIN, ft_program.actor_id())
        .check(ft_program.actor_id());
    market
        .add_nft_contract(ADMIN, nft_program.actor_id())
        .check(nft_program.actor_id());
    (ft_program, nft_program, market)
}

pub trait Program {
    fn inner_program(&self) -> &InnerProgram;

    fn actor_id(&self) -> ActorId {
        let bytes: [u8; 32] = self.inner_program().id().into();
        bytes.into()
    }
}

pub struct MetaStateReply<T>(pub T);

impl<T: Debug + PartialEq> MetaStateReply<T> {
    #[track_caller]
    pub fn check(self, value: T) {
        assert_eq!(self.0, value);
    }
}

pub struct Action<T, R>(pub RunResult, pub fn(T) -> R);

impl<T, R> Action<T, R> {
    #[track_caller]
    pub fn check(self, value: T)
    where
        R: Encode,
    {
        assert!(self.0.contains(&Log::builder().payload(self.1(value))));
    }

    #[track_caller]
    pub fn failed(self) {
        assert!(self.0.main_failed())
    }
}
