#![no_std]

use gear_lib::non_fungible_token::{io::NFTTransfer, nft_core::*, state::*, token::*};
use gear_lib_derive::{NFTCore, NFTMetaState, NFTStateKeeper};
use gstd::{exec, msg, prelude::*, ActorId};
use nft_io::{NFTAction, NFTEvent};
use primitive_types::{H256, U256};

const DELAY: u32 = 600_000;

#[derive(Debug, Default, NFTStateKeeper, NFTCore, NFTMetaState)]
pub struct NFT {
    #[NFTStateField]
    pub token: NFTState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub transactions: BTreeSet<H256>,
}

pub trait MyNFTCore: NFTCore {
    fn mint(&mut self, token_metadata: TokenMetadata) -> NFTTransfer;
}

impl MyNFTCore for NFT {
    fn mint(&mut self, token_metadata: TokenMetadata) -> NFTTransfer {
        let transfer = NFTCore::mint(self, &msg::source(), self.token_id, Some(token_metadata));
        self.token_id = self.token_id.saturating_add(U256::one());
        transfer
    }
}

impl NFT {
    pub fn transaction_made(&mut self, transaction_id: u64) -> bool {
        let transaction_hash = get_hash(&msg::source(), transaction_id);
        send_delayed_clear(transaction_hash);
        if self.transactions.insert(transaction_hash) {
            false
        } else {
            msg::reply(NFTEvent::TransactionMade, 0)
                .expect("Error during replying with `NFTEvent::TransactionMade`");
            true
        }
    }

    pub fn clear(&mut self, transaction_hash: H256) {
        assert_eq!(
            msg::source(),
            exec::program_id(),
            "Not allowed to clear transactions"
        );
        self.transactions.remove(&transaction_hash);
    }
}

pub fn get_hash(account: &ActorId, transaction_id: u64) -> H256 {
    let account: [u8; 32] = (*account).into();
    let transaction_id = transaction_id.to_be_bytes();
    sp_core_hashing::blake2_256(&[account.as_slice(), transaction_id.as_slice()].concat()).into()
}

fn send_delayed_clear(transaction_hash: H256) {
    msg::send_delayed(
        exec::program_id(),
        NFTAction::Clear { transaction_hash },
        0,
        DELAY,
    )
    .expect("Error in sending a delayed message `FTStorageAction::Clear`");
}
