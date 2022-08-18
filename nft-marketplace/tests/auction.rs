pub mod utils;
use utils::prelude::*;

#[test]
fn auction_with_native_tokens() {
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

    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            None,
            NFT_PRICE,
            BID_PERIOD,
            DURATION,
        )
        .check((nft_program.actor_id(), TOKEN_ID.into(), NFT_PRICE));

    for (i, &participant) in PARTICIPANTS.iter().enumerate() {
        let bid_price = (i as u128 + 2) * NFT_PRICE;
        system.mint_to(participant, bid_price);
        market
            .add_bid(
                participant,
                nft_program.actor_id(),
                TOKEN_ID.into(),
                bid_price,
                bid_price,
            )
            .check((nft_program.actor_id(), TOKEN_ID.into(), bid_price));

        // check that marketplace has returned funds to the previous participant
        if i != 0 {
            system.claim_value_from_mailbox(PARTICIPANTS[i - 1]);
            assert_eq!(
                system.balance_of(PARTICIPANTS[i - 1]),
                (i as u128 + 1) * NFT_PRICE
            );
        }
    }

    let winner_price = 6 * NFT_PRICE;
    let winner = PARTICIPANTS[4];

    // check balance of nft marketplace contract
    assert_eq!(system.balance_of(MARKET_ID), winner_price);

    // check item state
    let mut item = Item {
        owner_id: SELLER.into(),
        ft_contract_id: None,
        price: None,
        auction: Some(Auction {
            bid_period: BID_PERIOD,
            started_at: system.block_timestamp(),
            ended_at: system.block_timestamp() + DURATION,
            current_price: winner_price,
            current_winner: winner.into(),
        }),
        offers: BTreeMap::new(),
    };

    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item.clone());

    system.spend_blocks((DURATION / 1000) as u32);

    market
        .settle_auction(SELLER, nft_program.actor_id(), TOKEN_ID.into())
        .check(MarketEvent::AuctionSettled {
            nft_contract_id: nft_program.actor_id(),
            winner: winner.into(),
            token_id: TOKEN_ID.into(),
            price: winner_price,
        });

    item.auction = None;
    item.owner_id = winner.into();

    // check item state
    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item);

    // check owner
    nft_program
        .meta_state()
        .owner_id(TOKEN_ID)
        .check(winner.into());

    let treasury_fee = winner_price * ((TREASURY_FEE * BASE_PERCENT) as u128) / 10_000u128;

    // check balance of SELLER
    system.claim_value_from_mailbox(SELLER);
    assert_eq!(system.balance_of(SELLER), winner_price - treasury_fee);

    // check balance of TREASURY_ID
    system.claim_value_from_mailbox(TREASURY_ID);
    assert_eq!(system.balance_of(TREASURY_ID), treasury_fee);
}

#[test]
fn cancelled_auction() {
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

    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            None,
            NFT_PRICE,
            BID_PERIOD,
            DURATION,
        )
        .check((nft_program.actor_id(), TOKEN_ID.into(), NFT_PRICE));

    system.spend_blocks((DURATION / 1000) as u32);

    market
        .settle_auction(SELLER, nft_program.actor_id(), TOKEN_ID.into())
        .check(MarketEvent::AuctionCancelled {
            nft_contract_id: nft_program.actor_id(),
            token_id: TOKEN_ID.into(),
        });
}

#[test]
fn auction_with_fungible_tokens() {
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

    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            Some(ft_program.actor_id()),
            NFT_PRICE,
            BID_PERIOD,
            DURATION,
        )
        .check((nft_program.actor_id(), TOKEN_ID.into(), NFT_PRICE));

    for (i, &participant) in PARTICIPANTS.iter().enumerate() {
        let bid_price = (i as u128 + 2) * NFT_PRICE;
        ft_program.approve(participant, market.actor_id(), bid_price);
        ft_program.mint(participant, bid_price);
        market
            .add_bid(
                participant,
                nft_program.actor_id(),
                TOKEN_ID.into(),
                bid_price,
                0,
            )
            .check((nft_program.actor_id(), TOKEN_ID.into(), bid_price));

        // check that marketplace has returned funds to the previous participant
        if i != 0 {
            ft_program
                .balance_of(PARTICIPANTS[i - 1])
                .check((i as u128 + 1) * NFT_PRICE);
        }
    }

    let winner_price = 6 * NFT_PRICE;
    let winner = PARTICIPANTS[4];

    // check balance of nft marketplace contract
    ft_program.balance_of(MARKET_ID).check(winner_price);

    // check item state
    let mut item = Item {
        owner_id: SELLER.into(),
        ft_contract_id: Some(ft_program.actor_id()),
        price: None,
        auction: Some(Auction {
            bid_period: BID_PERIOD,
            started_at: system.block_timestamp(),
            ended_at: system.block_timestamp() + DURATION,
            current_price: winner_price,
            current_winner: winner.into(),
        }),
        offers: BTreeMap::new(),
    };

    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item.clone());

    system.spend_blocks((DURATION / 1000) as u32);

    market
        .settle_auction(SELLER, nft_program.actor_id(), TOKEN_ID.into())
        .check(MarketEvent::AuctionSettled {
            nft_contract_id: nft_program.actor_id(),
            winner: winner.into(),
            token_id: TOKEN_ID.into(),
            price: winner_price,
        });

    item.auction = None;
    item.owner_id = winner.into();

    // check item state
    market
        .meta_state()
        .item_info(nft_program.actor_id(), TOKEN_ID.into())
        .check(item);

    // check owner
    nft_program
        .meta_state()
        .owner_id(TOKEN_ID)
        .check(winner.into());

    let treasury_fee = winner_price * ((TREASURY_FEE * BASE_PERCENT) as u128) / 10_000u128;

    // check balance of SELLER
    ft_program
        .balance_of(SELLER)
        .check(winner_price - treasury_fee);

    // check balance of TREASURY_ID
    ft_program.balance_of(TREASURY_ID).check(treasury_fee);
}

#[test]
fn auction_failures() {
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

    // create auction failures

    // must fail since the bid period is less than 1 minute
    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            None,
            NFT_PRICE,
            MIN_BID_PERIOD - 100,
            DURATION,
        )
        .failed();

    // must fail since the bid period is less than 1 minute
    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            None,
            NFT_PRICE,
            BID_PERIOD,
            MIN_BID_PERIOD - 100,
        )
        .failed();

    // must fail since the min price is equal to zero
    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            None,
            0,
            BID_PERIOD,
            DURATION,
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
        .check((nft_program.actor_id(), TOKEN_ID.into(), NFT_PRICE));

    // must fail since the auction is already on
    market
        .create_auction(
            SELLER,
            (nft_program.actor_id(), TOKEN_ID.into()),
            None,
            NFT_PRICE,
            BID_PERIOD,
            DURATION,
        )
        .failed();

    // add bid and create auction failures

    // must fail since the price is equal to the current bid price
    system.mint_to(BUYER, NFT_PRICE);
    market
        .add_bid(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            NFT_PRICE,
            NFT_PRICE,
        )
        .failed();
    // must fail since the auction is not over
    market
        .settle_auction(SELLER, nft_program.actor_id(), TOKEN_ID.into())
        .failed();

    system.spend_blocks((DURATION as u32) / 1000);

    // must fail since the auction has alredy ended
    market
        .add_bid(
            BUYER,
            nft_program.actor_id(),
            TOKEN_ID.into(),
            NFT_PRICE,
            NFT_PRICE,
        )
        .failed();

    market
        .settle_auction(SELLER, nft_program.actor_id(), TOKEN_ID.into())
        .check(MarketEvent::AuctionCancelled {
            nft_contract_id: nft_program.actor_id(),
            token_id: TOKEN_ID.into(),
        });

    // must fail since the auction doesn't exist
    market
        .settle_auction(SELLER, nft_program.actor_id(), TOKEN_ID.into())
        .failed();
}
