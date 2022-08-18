use crate::{
    assert_auction_is_on, get_item, nft_messages::*, payment::*, ContractId, Market, MarketEvent,
    TokenId, BASE_PERCENT,
};
use gstd::{msg, prelude::*};

impl Market {
    pub async fn buy_item(&mut self, nft_contract_id: ContractId, token_id: TokenId) {
        let item = get_item(&mut self.items, nft_contract_id, token_id);
        assert_auction_is_on(&item.auction);
        let price = item.price.expect("The item is not on sale");

        check_attached_value(item.ft_contract_id, price);
        // fee for treasury
        let treasury_fee =
            price * (self.treasury_fee as u16 * BASE_PERCENT as u16) as u128 / 10_000u128;

        transfer_payment(
            msg::source(),
            self.treasury_id,
            item.ft_contract_id,
            treasury_fee,
        )
        .await;

        // transfer NFT and pay royalties
        let payouts = nft_transfer(
            nft_contract_id,
            msg::source(),
            token_id,
            price - treasury_fee,
        )
        .await;
        for (account, amount) in payouts.iter() {
            transfer_payment(msg::source(), *account, item.ft_contract_id, *amount).await;
        }

        item.owner_id = msg::source();
        item.price = None;

        msg::reply(
            MarketEvent::ItemSold {
                owner: msg::source(),
                nft_contract_id,
                token_id,
            },
            0,
        )
        .expect("Error in reply [MarketEvent::ItemSold]");
    }
}
