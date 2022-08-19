use super::{prelude::*, Action};
use ft_io::{FTAction, FTEvent, InitConfig as InitFT};
use gstd::ActorId;
use gtest::{Log, Program as InnerProgram, System};

pub struct FungibleToken<'a>(InnerProgram<'a>);

impl Program for FungibleToken<'_> {
    fn inner_program(&self) -> &InnerProgram {
        &self.0
    }
}

impl<'a> FungibleToken<'a> {
    pub fn initialize(system: &'a System) -> Self {
        let program = InnerProgram::from_file(system, "../target/fungible_token-0.1.0.wasm");

        assert!(!program
            .send(
                ADMIN,
                InitFT {
                    name: Default::default(),
                    symbol: Default::default(),
                    decimals: Default::default()
                }
            )
            .main_failed());

        Self(program)
    }

    pub fn mint(&self, from: u64, amount: u128) {
        assert!(self
            .0
            .send(from, FTAction::Mint(amount))
            .contains(&Log::builder().payload(FTEvent::Transfer {
                amount,
                from: ActorId::zero(),
                to: from.into()
            })));
    }

    pub fn approve(&self, from: u64, to: ActorId, amount: u128) {
        assert!(self
            .0
            .send(from, FTAction::Approve { to, amount })
            .contains(&Log::builder().payload(FTEvent::Approve {
                from: from.into(),
                to,
                amount,
            })));
    }
    pub fn balance_of(&self, actor_id: impl Into<ActorId>) -> Action<u128, FTEvent> {
        Action(
            self.0.send(ADMIN, FTAction::BalanceOf(actor_id.into())),
            FTEvent::Balance,
        )
    }
}
