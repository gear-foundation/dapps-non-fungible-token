pub mod utils;
use gstd::ActorId;
use utils::prelude::*;

#[test]
fn offers() {
    let system = utils::initialize_system();

    let (ft_program, nft_program, market) = utils::initialize_programs(&system);

    market
        .add_market_data(
            SELLER,
            nft_program.actor_id(),
            Some(ft_program.actor_id()),
            TOKEN_ID.into(),
            Some(NFT_PRICE),
        )
        .check((
            nft_program.actor_id(),
            Some(ft_program.actor_id()),
            TOKEN_ID.into(),
            Some(NFT_PRICE),
        ));

    let mut offers: BTreeMap<(Option<ContractId>, Price), ActorId> = BTreeMap::new();
    for i in 0..10 {
        let offered_price = 10_000 * (i + 1) as u128;
        system.mint_to(BUYER, offered_price);
        market
            .add_offer(
                BUYER,
                nft_program.actor_id(),
                TOKEN_ID.into(),
                None,
                offered_price,
                offered_price,
            )
            .check((nft_program.actor_id(), None, TOKEN_ID.into(), offered_price));
        offers.insert((None, offered_price), BUYER.into());
    }

    for i in 10..19 {
        let offered_price = 10_000 * (i + 1) as u128;
        ft_program.mint(BUYER, offered_price);
        ft_program.approve(BUYER, market.actor_id(), offered_price);
        market
            .add_offer(
                BUYER,
                nft_program.actor_id(),
                TOKEN_ID.into(),
                Some(ft_program.actor_id()),
                offered_price,
                0,
            )
            .check((
                nft_program.actor_id(),
                Some(ft_program.actor_id()),
                TOKEN_ID.into(),
                offered_price,
            ));
        offers.insert((Some(ft_program.actor_id()), offered_price), BUYER.into());
    }

    // check item state
    let mut item = Item {
        owner_id: SELLER.into(),
        ft_contract_id: Some(ft_program.actor_id()),
        price: Some(NFT_PRICE),
        auction: None,
        offers,
    };
    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item.clone());

    // accept offer (for fungible tokens)
    let accepted_price = 10_000 * 15;
    market
        .accept_offer(
            SELLER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            Some(ft_program.actor_id()),
            accepted_price,
        )
        .check((
            nft_program.actor_id(),
            Some(ft_program.actor_id()),
            TOKEN_ID.into(),
            BUYER.into(),
            accepted_price,
        ));

    // check owner
    nft_program
        .meta_state()
        .owner_id(TOKEN_ID)
        .check(BUYER.into());

    let treasury_fee = accepted_price * ((TREASURY_FEE * BASE_PERCENT) as u128) / 10_000u128;

    // check balance of SELLER
    ft_program
        .balance_of(SELLER)
        .check(accepted_price - treasury_fee);

    // check balance of TREASURY_ID
    ft_program.balance_of(TREASURY_ID).check(treasury_fee);

    // check item state
    item.owner_id = BUYER.into();
    item.price = None;
    item.offers
        .remove(&(Some(ft_program.actor_id()), accepted_price));
    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item.clone());

    // withdraw tokens
    let withdrawn_tokens = 10_000 * 12_u128;
    market
        .withdraw(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            Some(ft_program.actor_id()),
            withdrawn_tokens,
        )
        .check((
            nft_program.actor_id(),
            TOKEN_ID.into(),
            Some(ft_program.actor_id()),
            withdrawn_tokens,
        ));

    // check balance of BUYER after tokens withdrawal
    ft_program.balance_of(BUYER).check(withdrawn_tokens);

    // check item state
    item.offers
        .remove(&(Some(ft_program.actor_id()), withdrawn_tokens));
    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item.clone());

    // withdraw native tokens
    let withdrawn_tokens = 10_000 * 2_u128;
    market
        .withdraw(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            withdrawn_tokens,
        )
        .check((
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            withdrawn_tokens,
        ));

    // check item state
    item.offers.remove(&(None, withdrawn_tokens));
    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item.clone());

    // check balance of SELLER after tokens withdrawal
    system.claim_value_from_mailbox(BUYER);
    assert_eq!(system.balance_of(BUYER), withdrawn_tokens);

    // previous owner makes offer for native value
    let offered_value = 1_000_000;
    let buyer_balance = system.balance_of(BUYER);
    let treasury_fee = offered_value * ((TREASURY_FEE * BASE_PERCENT) as u128) / 10_000u128;
    system.mint_to(SELLER, offered_value);
    market
        .add_offer(
            SELLER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            offered_value,
            offered_value,
        )
        .check((nft_program.actor_id(), None, TOKEN_ID.into(), offered_value));

    // new owner accepts offer
    market
        .accept_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            offered_value,
        )
        .check((
            nft_program.actor_id(),
            None,
            TOKEN_ID.into(),
            SELLER.into(),
            offered_value,
        ));

    // check balance of BUYER
    system.claim_value_from_mailbox(BUYER);
    assert_eq!(
        system.balance_of(BUYER),
        buyer_balance + offered_value - treasury_fee
    );

    // check balance of TREASURY_ID
    system.claim_value_from_mailbox(TREASURY_ID);
    assert_eq!(system.balance_of(TREASURY_ID), treasury_fee);
}

#[test]
fn offers_failures() {
    let system = utils::initialize_system();

    let (ft_program, nft_program, market) = utils::initialize_programs(&system);

    market
        .add_market_data(
            SELLER,
            nft_program.actor_id(),
            None,
            TOKEN_ID.into(),
            Some(NFT_PRICE),
        )
        .check((
            nft_program.actor_id(),
            None,
            TOKEN_ID.into(),
            Some(NFT_PRICE),
        ));

    // must fail since the fungible token contract is not approved
    market
        .add_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            Some(ft_program.actor_id()),
            NFT_PRICE,
            0,
        )
        .failed();

    market
        .add_ft_contract(ADMIN, ft_program.actor_id())
        .check(ft_program.actor_id());

    // must fail since the price is zero
    market
        .add_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            Some(ft_program.actor_id()),
            0,
            0,
        )
        .failed();

    system.mint_to(BUYER, 2 * NFT_PRICE);

    // must fail since the attached value is not equal to the offered price
    market
        .add_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            NFT_PRICE,
            NFT_PRICE - 1000,
        )
        .failed();

    // add offer
    market
        .add_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            NFT_PRICE,
            NFT_PRICE,
        )
        .check((nft_program.actor_id(), None, TOKEN_ID.into(), NFT_PRICE));

    // must fail since the offers with these params already exists
    market
        .add_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            NFT_PRICE,
            NFT_PRICE,
        )
        .failed();

    // accept offer

    // must fail since only owner can accept offer
    market
        .accept_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            NFT_PRICE,
        )
        .failed();

    // must fail since the offer with the params doesn't exist
    market
        .accept_offer(
            SELLER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            2 * NFT_PRICE,
        )
        .failed();

    // start auction
    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            None,
            NFT_PRICE,
            BID_PERIOD,
            DURATION,
        )
        .check((nft_program.actor_id(), None, TOKEN_ID.into(), NFT_PRICE));

    // must fail since auction is on
    market
        .add_offer(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            NFT_PRICE - 1000,
            NFT_PRICE - 1000,
        )
        .failed();

    market
        .accept_offer(
            SELLER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            NFT_PRICE,
        )
        .failed();

    // withdraw failures

    // must fail since the caller isn't the offer author
    market
        .withdraw(
            SELLER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            NFT_PRICE,
        )
        .failed();

    // must fail since the indicated offer hash doesn't exist
    market
        .withdraw(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            None,
            2 * NFT_PRICE,
        )
        .failed();
}
