pub mod utils;
use utils::prelude::*;

#[test]
fn buy_with_fungible_tokens() {
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

    ft_program.mint(BUYER, NFT_PRICE);

    market
        .buy_item(BUYER, nft_program.actor_id(), TOKEN_ID.into(), 0)
        .check((nft_program.actor_id(), TOKEN_ID.into(), NFT_PRICE));

    // check owner
    nft_program
        .meta_state()
        .owner_id(TOKEN_ID)
        .check(BUYER.into());
    let treasury_fee = NFT_PRICE * ((TREASURY_FEE * BASE_PERCENT) as u128) / 10_000u128;

    // check balance of SELLER
    ft_program
        .balance_of(SELLER)
        .check(NFT_PRICE - treasury_fee);

    // check balance of TREASURY_ID
    ft_program.balance_of(TREASURY_ID).check(treasury_fee);
}

#[test]
fn buy_with_fungible_tokens_failures() {
    let system = utils::initialize_system();

    let (ft_program, nft_program, market) = utils::initialize_programs(&system);

    // must fail since item does not exist
    market
        .buy_item(BUYER, nft_program.actor_id(), TOKEN_ID.into(), 0)
        .failed();

    market
        .add_market_data(
            SELLER,
            nft_program.actor_id(),
            Some(ft_program.actor_id()),
            TOKEN_ID.into(),
            None,
        )
        .check((
            nft_program.actor_id(),
            Some(ft_program.actor_id()),
            TOKEN_ID.into(),
            None,
        ));

    // must fail since item isn't on sale
    market
        .buy_item(BUYER, nft_program.actor_id(), TOKEN_ID.into(), 0)
        .failed();

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

    // must fail since auction has started on that item
    market
        .buy_item(BUYER, nft_program.actor_id(), TOKEN_ID.into(), 0)
        .failed();
}

#[test]
fn buy_with_native_tokens() {
    let system = utils::initialize_system();

    let (_, nft_program, market) = utils::initialize_programs(&system);

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

    system.mint_to(BUYER, NFT_PRICE);

    // must fail since not enough value was attached to the message
    market
        .buy_item(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            NFT_PRICE - 1000,
        )
        .failed();

    market
        .buy_item(BUYER, nft_program.actor_id(), TOKEN_ID.into(), NFT_PRICE)
        .check((nft_program.actor_id(), TOKEN_ID.into(), NFT_PRICE));

    // check owner
    nft_program
        .meta_state()
        .owner_id(TOKEN_ID)
        .check(BUYER.into());

    let treasury_fee = NFT_PRICE * ((TREASURY_FEE * BASE_PERCENT) as u128) / 10_000u128;

    // check balance of SELLER
    system.claim_value_from_mailbox(SELLER);
    assert_eq!(system.balance_of(SELLER), NFT_PRICE - treasury_fee);

    // check balance of TREASURY_ID
    system.claim_value_from_mailbox(TREASURY_ID);
    assert_eq!(system.balance_of(TREASURY_ID), treasury_fee);
}
