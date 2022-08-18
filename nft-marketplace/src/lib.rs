#![no_std]

use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId};
pub use market_io::*;
use scale_info::TypeInfo;
pub mod nft_messages;
use nft_messages::*;
pub mod auction;
pub mod offers;
pub mod payment;
pub mod sale;
pub mod state;
use state::*;

const MIN_TREASURY_FEE: u8 = 0;
const MAX_TREASURY_FEE: u8 = 5;
pub const BASE_PERCENT: u16 = 100;

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
pub struct Market {
    pub admin_id: ActorId,
    pub treasury_id: ActorId,
    pub treasury_fee: u16,
    pub items: BTreeMap<(ContractId, TokenId), Item>,
    pub approved_nft_contracts: BTreeSet<ContractId>,
    pub approved_ft_contracts: BTreeSet<ContractId>,
}

static mut MARKET: Option<Market> = None;

impl Market {
    fn add_nft_contract(&mut self, nft_contract_id: ContractId) {
        self.check_admin();
        self.approved_nft_contracts.insert(nft_contract_id);
        msg::reply(MarketEvent::NftContractAdded(nft_contract_id), 0)
            .expect("Error in reply `MarketEvent::NftContractAdded`");
    }

    fn add_ft_contract(&mut self, ft_contract_id: ContractId) {
        self.check_admin();
        self.approved_ft_contracts.insert(ft_contract_id);
        msg::reply(MarketEvent::FtContractAdded(ft_contract_id), 0)
            .expect("Error in reply `MarketEvent::FtContractAdded`");
    }

    pub async fn add_market_data(
        &mut self,
        nft_contract_id: ContractId,
        ft_contract_id: Option<ActorId>,
        token_id: TokenId,
        price: Option<u128>,
    ) {
        self.check_approved_nft_contract(nft_contract_id);
        self.check_approved_ft_contract(ft_contract_id);
        let contract_and_token_id = (nft_contract_id, token_id);

        if let Some(item) = self.items.get_mut(&contract_and_token_id) {
            assert_auction_is_on(&item.auction);
            item.price = price;
            item.ft_contract_id = ft_contract_id
        } else {
            self.items.insert(
                contract_and_token_id,
                Item {
                    owner_id: msg::source(),
                    ft_contract_id,
                    price,
                    auction: None,
                    offers: BTreeMap::new(),
                },
            );
        }

        nft_approve(nft_contract_id, exec::program_id(), token_id).await;

        msg::reply(
            MarketEvent::MarketDataAdded {
                nft_contract_id,
                ft_contract_id,
                token_id,
                price,
            },
            0,
        )
        .expect("Error in reply [MarketEvent::MarketDataAdded]");
    }

    pub fn check_admin(&self) {
        assert!(
            msg::source() == self.admin_id,
            "Only owner can make that action"
        );
    }

    pub fn check_approved_nft_contract(&self, nft_contract_id: ContractId) {
        assert!(
            self.approved_nft_contracts.contains(&nft_contract_id),
            "that NFT contract is not approved"
        );
    }

    pub fn check_approved_ft_contract(&self, ft_contract_id: Option<ActorId>) {
        if ft_contract_id.is_some()
            && !self
                .approved_ft_contracts
                .contains(&ft_contract_id.expect("Must not be an error here"))
        {
            panic!("that FT contract is not approved");
        }
    }
}

pub fn get_item(
    items: &mut BTreeMap<(ContractId, TokenId), Item>,
    nft_contract_id: ContractId,
    token_id: TokenId,
) -> &mut Item {
    let contract_and_token_id = (nft_contract_id, token_id);
    items
        .get_mut(&contract_and_token_id)
        .expect("Item does not exist")
}

pub fn assert_auction_is_on(auction: &Option<Auction>) {
    assert!(auction.is_none(), "There is an opened auction");
}

#[gstd::async_main]
async unsafe fn main() {
    let action: MarketAction = msg::load().expect("Could not load Action");
    let market: &mut Market = unsafe { MARKET.get_or_insert(Market::default()) };
    match action {
        MarketAction::AddNftContract(nft_contract_id) => {
            market.add_nft_contract(nft_contract_id);
        }
        MarketAction::AddFTContract(ft_contract_id) => {
            market.add_ft_contract(ft_contract_id);
        }
        MarketAction::AddMarketData {
            nft_contract_id,
            ft_contract_id,
            token_id,
            price,
        } => {
            market
                .add_market_data(nft_contract_id, ft_contract_id, token_id, price)
                .await;
        }
        MarketAction::BuyItem {
            nft_contract_id,
            token_id,
        } => {
            market.buy_item(nft_contract_id, token_id).await;
        }
        MarketAction::Item {
            nft_contract_id,
            token_id,
        } => {
            let contract_and_token_id = (nft_contract_id, token_id);
            let item = market
                .items
                .get(&contract_and_token_id)
                .expect("Item does not exist")
                .clone();
            msg::reply(MarketEvent::ItemInfo(item), 0)
                .expect("Error in reply [MarketEvent::ItemInfo]");
        }
        MarketAction::AddOffer {
            nft_contract_id,
            ft_contract_id,
            token_id,
            price,
        } => {
            market
                .add_offer(nft_contract_id, ft_contract_id, token_id, price)
                .await
        }
        MarketAction::AcceptOffer {
            nft_contract_id,
            token_id,
            ft_contract_id,
            price,
        } => {
            market
                .accept_offer(nft_contract_id, token_id, ft_contract_id, price)
                .await
        }
        MarketAction::Withdraw {
            nft_contract_id,
            token_id,
            ft_contract_id,
            price,
        } => {
            market
                .withdraw(nft_contract_id, token_id, ft_contract_id, price)
                .await
        }
        MarketAction::CreateAuction {
            nft_contract_id,
            ft_contract_id,
            token_id,
            min_price,
            bid_period,
            duration,
        } => {
            market
                .create_auction(
                    nft_contract_id,
                    ft_contract_id,
                    token_id,
                    min_price,
                    bid_period,
                    duration,
                )
                .await;
        }
        MarketAction::AddBid {
            nft_contract_id,
            token_id,
            price,
        } => market.add_bid(nft_contract_id, token_id, price).await,

        MarketAction::SettleAuction {
            nft_contract_id,
            token_id,
        } => {
            market.settle_auction(nft_contract_id, token_id).await;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitMarket = msg::load().expect("Unable to decode InitConfig");
    if config.treasury_fee == MIN_TREASURY_FEE as u16
        || config.treasury_fee > MAX_TREASURY_FEE as u16
    {
        panic!("Wrong treasury fee");
    }
    let market = Market {
        admin_id: config.admin_id,
        treasury_id: config.treasury_id,
        treasury_fee: config.treasury_fee,
        ..Default::default()
    };
    MARKET = Some(market);
}

gstd::metadata! {
title: "NFTMarketplace",
    init:
        input: InitMarket,
    handle:
        input: MarketAction,
        output: MarketEvent,
    state:
        input: State,
        output: StateReply,
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let state: State = msg::load().expect("failed to decode input argument");
    let market: &mut Market = MARKET.get_or_insert(Market::default());
    let encoded = match state {
        State::AllItems => StateReply::AllItems(market.items.values().cloned().collect()).encode(),
        State::ItemInfo {
            nft_contract_id,
            token_id,
        } => {
            let contract_and_token_id = (nft_contract_id, token_id);
            if let Some(item) = market.items.get(&contract_and_token_id) {
                StateReply::ItemInfo(item.clone()).encode()
            } else {
                StateReply::ItemInfo(Item::default()).encode()
            }
        }
    };
    gstd::util::to_leak_ptr(encoded)
}
