use gear_lib::non_fungible_token::token::*;
use gstd::prelude::*;
use gtest::{Program, RunResult, System};
use on_chain_nft_io::*;
const USERS: &[u64] = &[3, 4, 5];

pub fn init_nft(sys: &System) {
    sys.init_logger();
    let nft = Program::current(sys);

    let mut layers = BTreeMap::new();
    let first_layer = BTreeMap::from([
        (
            1,
            String::from(
                "<svg height='210' width='500'><polygon points='100,10 40,198 190,78 10,78 160,198' style='fill:lime;stroke:purple;stroke-width:5;fill-rule:nonzero;'/></svg>",
            )
        ),
        (
            2,
            String::from(
                "<svg height='210' width='500'><polygon points='100,10 40,198 190,78 10,78 160,198' style='fill:blue;stroke:red;stroke-width:5;fill-rule:nonzero;'/></svg>",
            )
        )
    ]);
    let second_layer = BTreeMap::from([
        (
            1,
            String::from(
                "<svg height='30' width='200'><text x='0' y='15' fill='red'>On Chain NFT</text></svg>"
            ),
        ),
        (
            2,
            String::from(
                "<svg height='30' width='200'><text x='0' y='15' fill='green'>On Chain NFT</text></svg>"
            )
        )
    ]);
    layers.insert(1, first_layer);
    layers.insert(2, second_layer);
    let res = nft.send(
        USERS[0],
        InitOnChainNFT {
            name: String::from("OnChainToken"),
            symbol: String::from("OCT"),
            base_uri: String::from(""),
            royalties: None,
            base_image: String::from("<svg height='100' width='100'><circle cx='50' cy='50' r='40' stroke='black' stroke-width='3' fill='red' /></svg>"),
            layers,
        },
    );

    assert!(res.log().is_empty());
}

pub fn mint(nft: &Program, member: u64, description: BTreeMap<LayerId, LayerItemId>) -> RunResult {
    nft.send(
        member,
        OnChainNFTAction::Mint {
            token_metadata: TokenMetadata {
                name: "CryptoKitty".to_string(),
                description: "Description".to_string(),
                media: "http://".to_string(),
                reference: "http://".to_string(),
            },
            description,
        },
    )
}

pub fn burn(nft: &Program, member: u64, token_id: u64) -> RunResult {
    nft.send(
        member,
        OnChainNFTAction::Burn {
            token_id: token_id.into(),
        },
    )
}

pub fn transfer(nft: &Program, from: u64, to: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        OnChainNFTAction::Transfer {
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn approve(nft: &Program, from: u64, to: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        OnChainNFTAction::Approve {
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn token_uri(nft: &Program, from: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        OnChainNFTAction::TokenURI {
            token_id: token_id.into(),
        },
    )
}
