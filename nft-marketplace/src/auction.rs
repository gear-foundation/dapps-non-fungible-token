use crate::{
    assert_auction_is_on, get_item,
    nft_messages::{nft_approve, nft_transfer},
    payment::{check_attached_value, transfer_payment},
    Market, MarketEvent, BASE_PERCENT,
};
use gstd::{exec, msg, prelude::*, ActorId};
use market_io::*;
const MIN_BID_PERIOD: u64 = 60_000;

impl Market {
    pub async fn create_auction(
        &mut self,
        nft_contract_id: ContractId,
        ft_contract_id: Option<ActorId>,
        token_id: TokenId,
        min_price: u128,
        bid_period: u64,
        duration: u64,
    ) {
        self.check_approved_nft_contract(nft_contract_id);
        self.check_approved_ft_contract(ft_contract_id);

        if bid_period < MIN_BID_PERIOD || duration < MIN_BID_PERIOD {
            panic!("bid period or auction duration can't be less than 1 minute");
        }
        if min_price == 0 {
            panic!("price can't be equal to zero");
        }

        let item = get_item(&mut self.items, nft_contract_id, token_id);
        assert_auction_is_on(&item.auction);
        // approve nft to trade on the marketplace

        nft_approve(nft_contract_id, exec::program_id(), token_id).await;

        let auction = Auction {
            bid_period,
            started_at: exec::block_timestamp(),
            ended_at: exec::block_timestamp() + duration,
            current_price: min_price,
            current_winner: ActorId::zero(),
        };

        item.price = None;
        item.auction = Some(auction);
        item.ft_contract_id = ft_contract_id;

        msg::reply(
            MarketEvent::AuctionCreated {
                nft_contract_id,
                ft_contract_id,
                token_id,
                price: min_price,
            },
            0,
        )
        .expect("Error in reply [MarketEvent::AuctionCreated]");
    }

    pub async fn settle_auction(&mut self, nft_contract_id: ContractId, token_id: TokenId) {
        let item = get_item(&mut self.items, nft_contract_id, token_id);

        let auction = item.auction.clone().expect("Auction doesn not exist");

        if auction.ended_at > exec::block_timestamp() {
            panic!("Auction is not over");
        }
        let winner = auction.current_winner;
        let price = auction.current_price;

        if winner == ActorId::zero() {
            item.auction = None;
            msg::reply(
                MarketEvent::AuctionCancelled {
                    nft_contract_id,
                    token_id,
                },
                0,
            )
            .expect("Error in reply [MarketEvent::AuctionCancelled]");
            return;
        }

        // fee for treasury
        let treasury_fee = price * (self.treasury_fee * BASE_PERCENT) as u128 / 10_000u128;
        transfer_payment(
            exec::program_id(),
            self.treasury_id,
            item.ft_contract_id,
            treasury_fee,
        )
        .await;

        // transfer NFT and pay royalties
        let payouts = nft_transfer(nft_contract_id, winner, token_id, price - treasury_fee).await;
        for (account, amount) in payouts.iter() {
            transfer_payment(exec::program_id(), *account, item.ft_contract_id, *amount).await;
        }

        item.owner_id = winner;
        item.auction = None;
        msg::reply(
            MarketEvent::AuctionSettled {
                nft_contract_id,
                token_id,
                price,
                new_owner: winner,
            },
            0,
        )
        .expect("Error in reply [MarketEvent::AuctionSettled]");
    }

    pub async fn add_bid(&mut self, nft_contract_id: ContractId, token_id: TokenId, price: u128) {
        let item = get_item(&mut self.items, nft_contract_id, token_id);

        let mut auction = item.auction.clone().expect("Auction doesn not exist");
        if auction.ended_at < exec::block_timestamp() {
            panic!("Auction has already ended");
        }

        check_attached_value(item.ft_contract_id, price);

        let previous_price = auction.current_price;
        let previous_winner = auction.current_winner;

        if price <= previous_price {
            panic!("Cant offer less or equal to the current bid price")
        }

        if auction.ended_at <= exec::block_timestamp() + auction.bid_period {
            auction.ended_at = exec::block_timestamp() + auction.bid_period;
        }

        auction.current_price = price;
        auction.current_winner = msg::source();
        item.auction = Some(auction);
        // transfer payment from the current account to the marketplace contract
        transfer_payment(
            msg::source(),
            exec::program_id(),
            item.ft_contract_id,
            price,
        )
        .await;

        if previous_winner != ActorId::zero() {
            // transfer payment back to the previous winner
            transfer_payment(
                exec::program_id(),
                previous_winner,
                item.ft_contract_id,
                previous_price,
            )
            .await;
        }

        msg::reply(
            MarketEvent::BidAdded {
                nft_contract_id,
                token_id,
                price,
            },
            0,
        )
        .expect("Error in reply [MarketEvent::BidAdded]");
    }
}
