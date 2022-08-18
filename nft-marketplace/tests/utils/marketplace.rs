use super::{prelude::*, Action, MetaStateReply};
use gstd::ActorId;
use gtest::{Program as InnerProgram, System};
use market_io::{InitMarket, MarketAction, MarketEvent};
use nft_marketplace::state::{State, StateReply};
type ActionMarket<T> = Action<T, MarketEvent>;

pub struct Market<'a>(InnerProgram<'a>);

impl Program for Market<'_> {
    fn inner_program(&self) -> &InnerProgram {
        &self.0
    }
}

impl<'a> Market<'a> {
    pub fn initialize(system: &'a System) -> Self {
        Self::initialize_custom(
            system,
            InitMarket {
                admin_id: ADMIN.into(),
                treasury_id: TREASURY_ID.into(),
                treasury_fee: TREASURY_FEE,
            },
        )
        .succeed()
    }

    pub fn initialize_custom(system: &System, config: InitMarket) -> MarketInit {
        let program = InnerProgram::current(system);

        let failed = program.send(ADMIN, config).main_failed();

        MarketInit(program, failed)
    }

    pub fn meta_state(&self) -> MarketMetaState {
        MarketMetaState(&self.0)
    }

    pub fn add_nft_contract(
        &self,
        from: u64,
        nft_contract_id: ActorId,
    ) -> ActionMarket<ContractId> {
        Action(
            self.0
                .send(from, MarketAction::AddNftContract(nft_contract_id)),
            MarketEvent::NftContractAdded,
        )
    }

    pub fn add_ft_contract(&self, from: u64, ft_contract_id: ActorId) -> ActionMarket<ContractId> {
        Action(
            self.0
                .send(from, MarketAction::AddFTContract(ft_contract_id)),
            MarketEvent::FtContractAdded,
        )
    }

    pub fn add_market_data(
        &self,
        from: u64,
        nft_contract_id: ActorId,
        ft_contract_id: Option<ActorId>,
        token_id: TokenId,
        price: Option<u128>,
    ) -> ActionMarket<(ContractId, Option<ContractId>, TokenId, Option<Price>)> {
        Action(
            self.0.send(
                from,
                MarketAction::AddMarketData {
                    nft_contract_id,
                    ft_contract_id,
                    token_id,
                    price,
                },
            ),
            |(nft_contract_id, ft_contract_id, token_id, price)| MarketEvent::MarketDataAdded {
                nft_contract_id,
                ft_contract_id,
                token_id,
                price,
            },
        )
    }

    pub fn buy_item(
        &self,
        from: u64,
        nft_contract_id: ActorId,
        token_id: TokenId,
        value: u128,
    ) -> ActionMarket<(ActorId, ContractId, TokenId)> {
        Action(
            self.0.send_with_value(
                from,
                MarketAction::BuyItem {
                    nft_contract_id,
                    token_id,
                },
                value,
            ),
            |(owner, nft_contract_id, token_id)| MarketEvent::ItemSold {
                owner,
                nft_contract_id,
                token_id,
            },
        )
    }

    pub fn add_offer(
        &self,
        from: u64,
        nft_contract_id: ContractId,
        token_id: TokenId,
        ft_contract_id: Option<ContractId>,
        price: Price,
        value: u128,
    ) -> ActionMarket<(ContractId, Option<ContractId>, TokenId, Price)> {
        Action(
            self.0.send_with_value(
                from,
                MarketAction::AddOffer {
                    nft_contract_id,
                    ft_contract_id,
                    token_id,
                    price,
                },
                value,
            ),
            |(nft_contract_id, ft_contract_id, token_id, price)| MarketEvent::OfferAdded {
                nft_contract_id,
                ft_contract_id,
                token_id,
                price,
            },
        )
    }

    pub fn accept_offer(
        &self,
        from: u64,
        nft_contract_id: ContractId,
        token_id: TokenId,
        ft_contract_id: Option<ContractId>,
        price: Price,
    ) -> ActionMarket<(ContractId, TokenId, ActorId, Price)> {
        Action(
            self.0.send(
                from,
                MarketAction::AcceptOffer {
                    nft_contract_id,
                    token_id,
                    ft_contract_id,
                    price,
                },
            ),
            |(nft_contract_id, token_id, new_owner, price)| MarketEvent::OfferAccepted {
                nft_contract_id,
                token_id,
                new_owner,
                price,
            },
        )
    }

    pub fn withdraw(
        &self,
        from: u64,
        nft_contract_id: ContractId,
        token_id: TokenId,
        ft_contract_id: Option<ContractId>,
        price: Price,
    ) -> ActionMarket<(ContractId, TokenId, Option<ActorId>, Price)> {
        Action(
            self.0.send(
                from,
                MarketAction::Withdraw {
                    nft_contract_id,
                    token_id,
                    ft_contract_id,
                    price,
                },
            ),
            |(nft_contract_id, token_id, ft_contract_id, price)| MarketEvent::TokensWithdrawn {
                nft_contract_id,
                token_id,
                ft_contract_id,
                price,
            },
        )
    }

    pub fn create_auction(
        &self,
        from: u64,
        (nft_contract_id, token_id): (ContractId, TokenId),
        ft_contract_id: Option<ContractId>,
        min_price: u128,
        bid_period: u64,
        duration: u64,
    ) -> ActionMarket<(ContractId, TokenId, Price)> {
        Action(
            self.0.send(
                from,
                MarketAction::CreateAuction {
                    nft_contract_id,
                    ft_contract_id,
                    token_id,
                    min_price,
                    bid_period,
                    duration,
                },
            ),
            |(nft_contract_id, token_id, price)| MarketEvent::AuctionCreated {
                nft_contract_id,
                token_id,
                price,
            },
        )
    }

    pub fn add_bid(
        &self,
        from: u64,
        nft_contract_id: ActorId,
        token_id: TokenId,
        price: u128,
        value: u128,
    ) -> ActionMarket<(ContractId, TokenId, Price)> {
        Action(
            self.0.send_with_value(
                from,
                MarketAction::AddBid {
                    nft_contract_id,
                    token_id,
                    price,
                },
                value,
            ),
            |(nft_contract_id, token_id, price)| MarketEvent::BidAdded {
                nft_contract_id,
                token_id,
                price,
            },
        )
    }

    pub fn settle_auction(
        &self,
        from: u64,
        nft_contract_id: ActorId,
        token_id: TokenId,
    ) -> ActionMarket<MarketEvent> {
        Action(
            self.0.send(
                from,
                MarketAction::SettleAuction {
                    nft_contract_id,
                    token_id,
                },
            ),
            |market_event| market_event,
        )
    }
}

pub struct MarketMetaState<'a>(&'a InnerProgram<'a>);

impl MarketMetaState<'_> {
    pub fn item_info(self, nft_contract_id: ContractId, token_id: TokenId) -> MetaStateReply<Item> {
        if let StateReply::ItemInfo(item) = self
            .0
            .meta_state(State::ItemInfo {
                nft_contract_id,
                token_id,
            })
            .unwrap()
        {
            MetaStateReply(item)
        } else {
            unreachable!();
        }
    }
}
pub struct MarketInit<'a>(InnerProgram<'a>, bool);

impl<'a> MarketInit<'a> {
    #[track_caller]
    pub fn failed(self) {
        assert!(self.1)
    }

    #[track_caller]
    pub fn succeed(self) -> Market<'a> {
        assert!(!self.1);
        Market(self.0)
    }
}