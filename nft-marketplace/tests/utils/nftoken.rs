use super::{prelude::*, MetaStateReply};
use gear_lib::non_fungible_token::{
    io::NFTEvent,
    state::{NFTQuery, NFTQueryReply},
    token::Token,
};
use gstd::ActorId;
use gtest::{Log, Program as InnerProgram, System};
use nft_io::{InitNFT, NFTAction};

pub struct NonFungibleToken<'a>(InnerProgram<'a>);

impl Program for NonFungibleToken<'_> {
    fn inner_program(&self) -> &InnerProgram {
        &self.0
    }
}

impl<'a> NonFungibleToken<'a> {
    pub fn initialize(system: &'a System) -> Self {
        let program =
            InnerProgram::from_file(system, "../target/wasm32-unknown-unknown/release/nft.wasm");

        assert!(!program
            .send(
                ADMIN,
                InitNFT {
                    name: Default::default(),
                    symbol: Default::default(),
                    base_uri: Default::default(),
                    royalties: Default::default()
                }
            )
            .main_failed());

        Self(program)
    }

    pub fn mint(&self, from: u64) {
        assert!(self
            .0
            .send(
                from,
                NFTAction::Mint {
                    token_metadata: Default::default()
                }
            )
            .contains(
                &Log::builder().payload(
                    NFTEvent::Minted {
                        owner: from.into(),
                        token_id: TOKEN_ID.into(),
                        token_metadata: Some(Default::default())
                    }
                )
            ));
    }

    pub fn meta_state(&self) -> NonFungibleTokenMetaState {
        NonFungibleTokenMetaState(&self.0)
    }
}

pub struct NonFungibleTokenMetaState<'a>(&'a InnerProgram<'a>);

impl NonFungibleTokenMetaState<'_> {
    pub fn owner_id(self, token_id: u64) -> MetaStateReply<ActorId> {
        MetaStateReply(self.token(token_id).0.owner_id)
    }

    pub fn token(self, token_id: u64) -> MetaStateReply<Token> {
        if let NFTQueryReply::Token { token: reply } = self
            .0
            .meta_state(NFTQuery::Token {
                token_id: token_id.into(),
            })
            .unwrap()
        {
            MetaStateReply(reply)
        } else {
            unreachable!();
        }
    }
}
