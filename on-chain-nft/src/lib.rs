#![no_std]

use gear_lib::non_fungible_token::{nft_core::*, state::*, token::*};
use gear_lib_derive::{NFTCore, NFTMetaState, NFTStateKeeper};
use gstd::{msg, prelude::*, ActorId};
use on_chain_nft_io::*;
use primitive_types::U256;

#[derive(Debug, Default, NFTStateKeeper, NFTCore, NFTMetaState)]
pub struct OnChainNFT {
    #[NFTStateField]
    pub token: NFTState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub base_image: Vec<u8>,
    pub layers: BTreeMap<LayerId, BTreeMap<ItemId, Vec<u8>>>,
    pub nfts: BTreeMap<TokenId, BTreeMap<LayerId, ItemId>>,
    pub nfts_existence: BTreeSet<String>,
}

static mut CONTRACT: Option<OnChainNFT> = None;

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitOnChainNFT = msg::load().expect("Unable to decode InitOnChainNFT");
    let mut _layers: BTreeMap<LayerId, BTreeMap<ItemId, Vec<u8>>> = BTreeMap::new();
    for (layer_id, layer) in config.layers.iter() {
        let mut layer_map: BTreeMap<ItemId, Vec<u8>> = BTreeMap::new();
        for (layer_item_id, layer_item) in layer.clone() {
            layer_map.insert(layer_item_id, layer_item.into_bytes());
        }

        _layers.insert(*layer_id, layer_map);
    }
    let nft = OnChainNFT {
        token: NFTState {
            name: config.name,
            symbol: config.symbol,
            base_uri: config.base_uri,
            ..Default::default()
        },
        owner: msg::source(),
        base_image: config.base_image.into_bytes(),
        layers: _layers,
        ..Default::default()
    };
    CONTRACT = Some(nft);
}

#[no_mangle]
pub unsafe extern "C" fn handle() {
    let action: OnChainNFTAction = msg::load().expect("Could not load OnChainNFTAction");
    let nft = CONTRACT.get_or_insert(Default::default());
    match action {
        OnChainNFTAction::Mint {
            description,
            token_metadata,
        } => OnChainNFTCore::mint(nft, description, token_metadata),
        OnChainNFTAction::Burn { token_id } => OnChainNFTCore::burn(nft, token_id),
        OnChainNFTAction::Transfer { to, token_id } => NFTCore::transfer(nft, &to, token_id),
        OnChainNFTAction::TransferPayout {
            to,
            token_id,
            amount,
        } => NFTCore::transfer_payout(nft, &to, token_id, amount),
        OnChainNFTAction::Approve { to, token_id } => NFTCore::approve(nft, &to, token_id),
    }
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: OnChainNFTQuery = msg::load().expect("failed to decode input argument");
    let nft = CONTRACT.get_or_insert(OnChainNFT::default());
    match query {
        OnChainNFTQuery::TokenURI { token_id } => {
            let encoded = OnChainNFTCore::token_uri(nft, token_id)
                .expect("Error in reading OnChainNFT contract state");
            gstd::util::to_leak_ptr(encoded)
        }
        OnChainNFTQuery::Base(query) => {
            let encoded =
                NFTMetaState::proc_state(nft, query).expect("Error in reading NFT contract state");
            gstd::util::to_leak_ptr(encoded)
        }
    }
}

pub trait OnChainNFTCore: NFTCore {
    fn mint(&mut self, description: BTreeMap<LayerId, ItemId>, metadata: TokenMetadata);
    fn burn(&mut self, token_id: TokenId);
    fn token_uri(&mut self, token_id: TokenId) -> Option<Vec<u8>>;
}

impl OnChainNFTCore for OnChainNFT {
    fn mint(&mut self, description: BTreeMap<LayerId, ItemId>, metadata: TokenMetadata) {
        // precheck if the layers actually exist
        for (layer_id, layer_item_id) in description.iter() {
            let _ = self
                .layers
                .get(layer_id)
                .expect("No such layer")
                .get(layer_item_id)
                .expect("No such layer item");
        }

        // also check if description has all layers provided
        if description.len() != self.layers.len() {
            panic!("The number of layers must be equal to the number of layers in the contract");
        }
        // precheck if there is already an nft with such description
        let mut key = String::from("");
        for lii in description.values() {
            key = key + &lii.to_string();
        }
        if self.nfts_existence.contains(&key) {
            panic!("Such nft already exists");
        }
        self.nfts_existence.insert(key);
        NFTCore::mint(self, &msg::source(), self.token_id, Some(metadata));
        self.nfts.insert(self.token_id, description);
        self.token_id = self.token_id.saturating_add(U256::one());
    }

    fn burn(&mut self, token_id: TokenId) {
        NFTCore::burn(self, token_id);
        self.nfts.remove(&token_id);
    }

    fn token_uri(&mut self, token_id: TokenId) -> Option<Vec<u8>> {
        let mut metadata = TokenMetadata::default();
        if let Some(Some(mtd)) = self.token.token_metadata_by_id.get(&token_id) {
            metadata = mtd.clone();
        }
        // construct media
        let mut content: Vec<String> = Vec::new();
        // check if exists
        let nft = self.nfts.get(&token_id).expect("No such nft");
        for (layer_id, layer_item_id) in nft {
            let layer_content = self
                .layers
                .get(layer_id)
                .expect("No such layer")
                .get(layer_item_id)
                .expect("No such layer item");
            let cc = String::from_utf8((*layer_content).clone()).expect("Found invalid UTF-8");
            content.push(cc);
        }
        Some(TokenURI { metadata, content }.encode())
    }
}

gstd::metadata! {
    title: "OnChainNFT",
    init:
        input: InitOnChainNFT,
    handle:
        input: OnChainNFTAction,
        output: Vec<u8>,
    state:
        input: OnChainNFTQuery,
        output: Vec<u8>,
}
