use gear_lib::non_fungible_token::token::*;
use gstd::ActorId;
use gtest::{Program, RunResult, System};
use nft_io::*;

const USERS: &[u64] = &[3, 4, 5];

pub fn init_nft(sys: &System) {
    sys.init_logger();
    let nft = Program::current(sys);

    let res = nft.send(
        USERS[0],
        InitNFT {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
            base_uri: String::from(""),
            royalties: None,
        },
    );

    assert!(res.log().is_empty());
}

pub fn mint(nft: &Program, transaction_id: u64, member: u64) -> RunResult {
    nft.send(
        member,
        NFTAction::Mint {
            transaction_id,
            token_metadata: TokenMetadata {
                name: "CryptoKitty".to_string(),
                description: "Description".to_string(),
                media: "http://".to_string(),
                reference: "http://".to_string(),
            },
        },
    )
}

pub fn burn(nft: &Program, transaction_id: u64, member: u64, token_id: u64) -> RunResult {
    nft.send(
        member,
        NFTAction::Burn {
            transaction_id,
            token_id: token_id.into(),
        },
    )
}

pub fn transfer(
    nft: &Program,
    transaction_id: u64,
    from: u64,
    to: u64,
    token_id: u64,
) -> RunResult {
    nft.send(
        from,
        NFTAction::Transfer {
            transaction_id,
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn owner_of(nft: &Program, from: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        NFTAction::Owner {
            token_id: token_id.into(),
        },
    )
}

pub fn update_user(
    nft: &Program,
    from: u64,
    address: ActorId,
    token_id: TokenId,
    expires: u64,
) -> RunResult {
    let payload = NFTAction::UpdateUser {
        token_id,
        address,
        expires,
    };
    nft.send(from, payload)
}

pub fn is_approved_to(nft: &Program, from: u64, token_id: u64, to: u64) -> RunResult {
    nft.send(
        from,
        NFTAction::IsApproved {
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn approve(nft: &Program, transaction_id: u64, from: u64, to: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        NFTAction::Approve {
            transaction_id,
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn delegated_approve(
    nft: &Program,
    transaction_id: u64,
    from: u64,
    message: DelegatedApproveMessage,
    signature: [u8; 64],
) -> RunResult {
    let action = NFTAction::DelegatedApprove {
        transaction_id,
        message,
        signature,
    };
    nft.send(from, action)
}

pub fn mint_to_actor(nft: &Program, transaction_id: u64, member: [u8; 32]) -> RunResult {
    nft.send(
        member,
        NFTAction::Mint {
            transaction_id,
            token_metadata: TokenMetadata {
                name: "CryptoKitty".to_string(),
                description: "Description".to_string(),
                media: "http://".to_string(),
                reference: "http://".to_string(),
            },
        },
    )
}

pub fn set_user(
    nft: &Program,
    from: u64,
    address: ActorId,
    token_id: TokenId,
    expires: u64,
) -> RunResult {
    let payload = NFTAction::SetUser {
        token_id,
        address,
        expires,
    };
    nft.send(from, payload)
}

pub fn user_of(nft: &Program, from: u64, token_id: TokenId) -> RunResult {
    let payload = NFTAction::UserOf { token_id };
    nft.send(from, payload)
}

pub fn user_expires(nft: &Program, from: u64, token_id: TokenId) -> RunResult {
    let payload = NFTAction::UserExpires { token_id };
    nft.send(from, payload)
}
