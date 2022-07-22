use crate::{nft_messages::*, payment::*, Market, BASE_PERCENT};
use gstd::{exec, msg, prelude::*, ActorId};
use market_io::*;
use primitive_types::H256;

pub fn get_hash(ft_contract_id: Option<ActorId>, price: u128) -> H256 {
    let price = price.to_be_bytes();
    let ft_id_vec: Vec<u8> = ft_contract_id
        .map(|id| <[u8; 32]>::from(id).into())
        .unwrap_or_default();
    sp_core_hashing::blake2_256(&[&ft_id_vec[..], &price[..]].concat()).into()
}

impl Market {
    pub async fn add_offer(
        &mut self,
        nft_contract_id: ContractId,
        ft_contract_id: Option<ContractId>,
        token_id: TokenId,
        price: Price,
    ) {
        let contract_and_token_id = (nft_contract_id, token_id);
        self.check_approved_ft_contract(ft_contract_id);
        self.on_auction(contract_and_token_id);
        let item = self
            .items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");
        if price == 0 {
            panic!("Cant offer zero price");
        }

        let offer = (ft_contract_id, price);
        let mut offers = item.offers.clone();
        if offers.insert(offer, msg::source()).is_some() {
            panic!("the offer with these params already exists");
        }

        check_attached_value(ft_contract_id, price);

        transfer_payment(msg::source(), exec::program_id(), ft_contract_id, price).await;

        item.offers = offers;
        msg::reply(
            MarketEvent::OfferAdded {
                nft_contract_id,
                ft_contract_id,
                token_id,
                price,
            },
            0,
        )
        .expect("Error in reply [MarketEvent::OfferAdded]");
    }

    /// Accepts an offer
    /// Requirements:
    /// * NFT item must be listed on the marketplace
    /// * Only owner can accept offer
    /// * There must be no ongoing auction
    /// * The offer with indicated hash must exist
    /// Arguments:
    /// * `nft_contract_id`: the NFT contract address
    /// * `token_id`: the NFT id
    /// * `offer_hash`: the offer hash
    pub async fn accept_offer(
        &mut self,
        nft_contract_id: ContractId,
        token_id: TokenId,
        ft_contract_id: Option<ContractId>,
        price: Price,
    ) {
        let contract_and_token_id = (nft_contract_id, token_id);
        self.on_auction(contract_and_token_id);
        let item = self
            .items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");
        if item.owner_id != msg::source() {
            panic!("only owner can accept offer");
        }
        let offer = (ft_contract_id, price);
        if let Some(id) = item.offers.remove(&offer) {
            let treasury_fee = price * (self.treasury_fee * BASE_PERCENT) as u128 / 10_000u128;
            transfer_payment(
                exec::program_id(),
                self.treasury_id,
                ft_contract_id,
                treasury_fee,
            )
            .await;
            // transfer NFT and pay royalties
            let payouts = nft_transfer(nft_contract_id, id, token_id, price - treasury_fee).await;
            for (account, amount) in payouts.iter() {
                transfer_payment(exec::program_id(), *account, ft_contract_id, *amount).await;
            }
            item.price = None;
            item.owner_id = id;
            msg::reply(
                MarketEvent::OfferAccepted {
                    nft_contract_id,
                    token_id,
                    new_owner: id,
                    price,
                },
                0,
            )
            .expect("Error in reply [MarketEvent::OfferAccepted]");
        } else {
            panic!("The offer does not exist");
        }
    }

    pub async fn withdraw(
        &mut self,
        nft_contract_id: ContractId,
        token_id: TokenId,
        ft_contract_id: Option<ContractId>,
        price: Price,
    ) {
        let contract_and_token_id = (nft_contract_id, token_id);
        let item = self
            .items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");

        let offer = (ft_contract_id, price);
        if let Some(id) = item.offers.remove(&offer) {
            if msg::source() != id {
                panic!("can't withdraw other user's tokens");
            }
            transfer_payment(exec::program_id(), msg::source(), ft_contract_id, price).await;
            msg::reply(
                MarketEvent::TokensWithdrawn {
                    nft_contract_id,
                    token_id,
                    ft_contract_id,
                    price,
                },
                0,
            )
            .expect("Error in reply [MarketEvent::TokensWithdrawn]");
        } else {
            panic!("The offer does not exist");
        }
    }
}
