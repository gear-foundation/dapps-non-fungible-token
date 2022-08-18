use codec::Encode;
use ft_io::*;
use gear_lib::non_fungible_token::token::TokenId;
use gstd::ActorId;
use gtest::{Program, System};
use market_io::*;
use nft_io::*;
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};

pub const USERS: Vec<[u8; 32]> = USERS_PAIRS
    .iter()
    .map(|pair| pair.public().0)
    .clone()
    .collect();
pub const USERS_PAIRS: &[Sr25519Pair] = &[
    Sr25519Pair::generate().0,
    Sr25519Pair::generate().0,
    Sr25519Pair::generate().0,
    Sr25519Pair::generate().0,
];
pub const TREASURY_ID: u64 = 8;

pub fn init_ft(sys: &System) {
    let ft = Program::from_file(sys, "../target/fungible_token-0.1.0.wasm");

    let res = ft.send(
        USERS[0],
        InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
            decimals: 18,
        },
    );

    assert!(res.log().is_empty());
}

pub fn init_nft(sys: &System) {
    sys.init_logger();
    let nft = Program::from_file(sys, "../target/wasm32-unknown-unknown/release/nft.wasm");

    let res = nft.send(
        USERS[0],
        InitNFT {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
            base_uri: "".to_string(),
            royalties: None,
        },
    );
    assert!(res.log().is_empty());
}

pub fn init_market(sys: &System) {
    sys.init_logger();
    let market = Program::current(sys);
    let res = market.send(
        USERS[0],
        InitMarket {
            admin_id: USERS[0].into(),
            treasury_id: TREASURY_ID.into(),
            treasury_fee: 1,
        },
    );
    assert!(res.log().is_empty());
}

pub fn add_market_data(
    sys: &System,
    market: &Program,
    ft_contract_id: Option<ActorId>,
    user_pair: Sr25519Pair,
    token_id: u128,
    price: Option<u128>,
) {
    // lists nft on the market
    let approve = create_delegated_approve(
        user_pair.clone(),
        ActorId::new(
            market
                .id()
                .as_ref()
                .try_into()
                .expect("slice with incorrect length"),
        ),
        2.into(),
        token_id.into(),
        sys.block_timestamp() + 100,
    );
    let res = market.send(
        user_pair.public().0,
        MarketAction::AddMarketData {
            delegated_approve: approve,
            ft_contract_id,
            price,
        },
    );
    assert!(res.contains(&(
        user_pair.public().0,
        MarketEvent::MarketDataAdded {
            nft_contract_id: 2.into(),
            owner: user_pair.public().0.into(),
            token_id: token_id.into(),
            price,
        }
        .encode()
    )));
}

fn create_delegated_approve(
    token_owner: Sr25519Pair,
    approved_actor_id: ActorId,
    nft_program_id: ActorId,
    token_id: TokenId,
    expiration_timestamp: u64,
) -> DelegatedApprove {
    let message = DelegatedApproveMessage {
        token_owner_id: token_owner.public().0.into(),
        approved_actor_id,
        nft_program_id,
        token_id,
        expiration_timestamp,
    };

    let signature = token_owner.sign(message.encode().as_slice()).0;

    DelegatedApprove { message, signature }
}
