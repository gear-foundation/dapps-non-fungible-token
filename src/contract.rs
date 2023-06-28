use gear_lib::non_fungible_token::{io::NFTTransfer, nft_core::*, state::*, token::*};
use gear_lib_derive::{NFTCore, NFTMetaState, NFTStateKeeper};
use gstd::{errors::Result as GstdResult, exec, msg, prelude::*, ActorId, MessageId};
use hashbrown::HashMap;
use nft_io::{Collection, Constraints, InitNFT, IoNFT, NFTAction, NFTEvent, Nft, Referral, State};
use primitive_types::{H256, U256};

#[derive(Debug, Default, NFTStateKeeper, NFTCore, NFTMetaState)]
pub struct Contract {
    #[NFTStateField]
    pub token: NFTState,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub transactions: HashMap<H256, NFTEvent>,
    pub collection: Collection,
    pub constraints: Constraints,
    pub referral_metadata: Vec<TokenMetadata>,
}

static mut CONTRACT: Option<Contract> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let config: InitNFT = msg::load().expect("Unable to decode InitNFT");
    if config.royalties.is_some() {
        config.royalties.as_ref().expect("Unable to g").validate();
    }
    let nft = Contract {
        token: NFTState {
            name: config.collection.name.clone(),
            royalties: config.royalties,
            ..Default::default()
        },
        collection: config.collection,
        constraints: config.constraints,
        owner: msg::source(),
        ..Default::default()
    };
    CONTRACT = Some(nft);
}

#[no_mangle]
unsafe extern "C" fn handle() {
    let action: NFTAction = msg::load().expect("Could not load NFTAction");
    let nft = CONTRACT.get_or_insert(Default::default());
    let msg_src = msg::source();
    match action {
        NFTAction::Mint {
            transaction_id,
            token_metadata,
        } => {
            if nft.is_authorized_minter(&msg_src) {
                msg::reply(
                    nft.process_transaction(transaction_id, |nft| {
                        NFTEvent::Transfer(MyNFTCore::mint(nft, token_metadata))
                    }),
                    0,
                )
                .expect("Error during replying with `NFTEvent::Transfer`");
            } else {
                panic!(
                    "Current ID {:?} is not authorized ar minter or referral",
                    msg_src
                );
            }
        }
        NFTAction::MintReferral { transaction_id } => {
            let index = random() as usize % nft.referral_metadata.len();
            let token_metadata = nft.referral_metadata.swap_remove(index);

            if nft.is_valid_referral(&msg_src) {
                msg::reply(
                    nft.process_transaction(transaction_id, |nft| {
                        NFTEvent::Transfer(MyNFTCore::mint(nft, token_metadata))
                    }),
                    0,
                )
                .expect("Error during replying with `NFTEvent::Transfer`");
                nft.constraints.referrals.insert(Referral { id: msg_src });
            } else {
                panic!("Current ID {:?} is not referral or already minted", msg_src);
            }
        }
        NFTAction::Burn {
            transaction_id,
            token_id,
        } => {
            msg::reply(
                nft.process_transaction(transaction_id, |nft| {
                    NFTEvent::Transfer(NFTCore::burn(nft, token_id))
                }),
                0,
            )
            .expect("Error during replying with `NFTEvent::Transfer`");
        }
        NFTAction::Transfer {
            transaction_id,
            to,
            token_id,
        } => {
            msg::reply(
                nft.process_transaction(transaction_id, |nft| {
                    NFTEvent::Transfer(NFTCore::transfer(nft, &to, token_id))
                }),
                0,
            )
            .expect("Error during replying with `NFTEvent::Transfer`");
        }
        NFTAction::TransferPayout {
            transaction_id,
            to,
            token_id,
            amount,
        } => {
            msg::reply(
                nft.process_transaction(transaction_id, |nft| {
                    NFTEvent::TransferPayout(NFTCore::transfer_payout(nft, &to, token_id, amount))
                }),
                0,
            )
            .expect("Error during replying with `NFTEvent::TransferPayout`");
        }
        NFTAction::NFTPayout { owner, amount } => {
            msg::reply(
                NFTEvent::NFTPayout(NFTCore::nft_payout(nft, &owner, amount)),
                0,
            )
            .expect("Error during replying with `NFTEvent::NFTPayout`");
        }
        NFTAction::Approve {
            transaction_id,
            to,
            token_id,
        } => {
            msg::reply(
                nft.process_transaction(transaction_id, |nft| {
                    NFTEvent::Approval(NFTCore::approve(nft, &to, token_id))
                }),
                0,
            )
            .expect("Error during replying with `NFTEvent::Approval`");
        }
        NFTAction::Owner { token_id } => {
            msg::reply(
                NFTEvent::Owner {
                    owner: NFTCore::owner_of(nft, token_id),
                    token_id,
                },
                0,
            )
            .expect("Error during replying with `NFTEvent::Owner`");
        }
        NFTAction::IsApproved { to, token_id } => {
            msg::reply(
                NFTEvent::IsApproved {
                    to,
                    token_id,
                    approved: NFTCore::is_approved_to(nft, &to, token_id),
                },
                0,
            )
            .expect("Error during replying with `NFTEvent::IsApproved`");
        }
        NFTAction::DelegatedApprove {
            transaction_id,
            message,
            signature,
        } => {
            msg::reply(
                nft.process_transaction(transaction_id, |nft| {
                    NFTEvent::Approval(NFTCore::delegated_approve(nft, message, signature))
                }),
                0,
            )
            .expect("Error during replying with `NFTEvent::Approval`");
        }
        NFTAction::Clear { transaction_hash } => nft.clear(transaction_hash),
        NFTAction::AddMinter {
            transaction_id,
            minter_id,
        } => {
            if nft.is_authorized_minter(&msg_src) {
                msg::reply(
                    nft.process_transaction(transaction_id, |nft| {
                        nft.constraints.authorized_minters.insert(minter_id);
                        NFTEvent::MinterAdded { minter_id }
                    }),
                    0,
                )
                .expect("Error during replying with `NFTEvent::Approval`");
            } else {
                panic!(
                    "Current ID {:?} is not authorized and does not have the right to add another ID as a minter",
                    msg_src
                );
            }
        }
        NFTAction::AddReferral {
            transaction_id,
            referral_id,
        } => {
            msg::reply(
                nft.process_transaction(transaction_id, |nft| {
                    let referral = Referral { id: referral_id };
                    nft.constraints.referrals.insert(referral);
                    NFTEvent::ReferralAdded { referral_id }
                }),
                0,
            )
            .expect("Error during replying with `NFTEvent::Approval`");
        }
        NFTAction::AddReferralMetadata(metadata) => {
            nft.referral_metadata.push(metadata);
        }
    };
}

pub trait MyNFTCore: NFTCore {
    fn mint(&mut self, token_metadata: TokenMetadata) -> NFTTransfer;
}

impl MyNFTCore for Contract {
    fn mint(&mut self, token_metadata: TokenMetadata) -> NFTTransfer {
        let transfer = NFTCore::mint(self, &msg::source(), self.token_id, Some(token_metadata));
        self.token_id = self.token_id.saturating_add(U256::one());
        transfer
    }
}

// Добавить загрузку ссылок на картинки, загрузить картинки в контракт.
// Выдача - посылается сообщения Mint без полей, контракт генерирует число 1-1000 и если такое число не выдано то выдаётся NFT под этим номером
// referrals can mint only 1 time то что прошел по реферальной ссылке, то что один раз сминтил, и выдать NFT которую
impl Contract {
    fn process_transaction(
        &mut self,
        transaction_id: u64,
        action: impl FnOnce(&mut Contract) -> NFTEvent,
    ) -> NFTEvent {
        let transaction_hash = get_hash(&msg::source(), transaction_id);

        if let Some(nft_event) = self.transactions.get(&transaction_hash) {
            nft_event.clone()
        } else {
            let nft_event = action(self);

            self.transactions
                .insert(transaction_hash, nft_event.clone());

            nft_event
        }
    }

    fn clear(&mut self, transaction_hash: H256) {
        assert_eq!(
            msg::source(),
            exec::program_id(),
            "Not allowed to clear transactions"
        );
        self.transactions.remove(&transaction_hash);
    }

    fn is_authorized_minter(&self, current_minter: &ActorId) -> bool {
        if let Some(max_mint_count) = self.constraints.max_mint_count {
            if max_mint_count <= self.token.token_metadata_by_id.len() as u32 {
                panic!(
                    "Mint impossible because max minting count {} limit exceeded",
                    max_mint_count
                );
            }
        }

        self.constraints
            .authorized_minters
            .iter()
            .any(|authorized_minter| authorized_minter.eq(current_minter))
    }

    fn is_valid_referral(&self, current_referral: &ActorId) -> bool {
        let is_exist = self
            .constraints
            .referrals
            .iter()
            .any(|referral| current_referral.eq(&referral.id));

        let can_mint = self.tokens_for_owner(current_referral).is_empty();

        is_exist && can_mint
    }
}

#[no_mangle]
extern "C" fn metahash() {
    let metahash: [u8; 32] = include!("../.metahash");
    reply(metahash).expect("Failed to encode or reply with `[u8; 32]` from `metahash()`");
}

fn static_mut_state() -> &'static Contract {
    unsafe { CONTRACT.get_or_insert(Default::default()) }
}

#[no_mangle]
extern "C" fn state() {
    let payload: nft_io::State = static_mut_state().into();
    reply(payload).expect("Unable to reply `nft_io::State`");
}

fn reply(payload: impl Encode) -> GstdResult<MessageId> {
    msg::reply(payload, 0)
}

pub fn get_hash(account: &ActorId, transaction_id: u64) -> H256 {
    let account: [u8; 32] = (*account).into();
    let transaction_id = transaction_id.to_be_bytes();
    sp_core_hashing::blake2_256(&[account.as_slice(), transaction_id.as_slice()].concat()).into()
}

impl From<&Contract> for IoNFT {
    fn from(value: &Contract) -> Self {
        let Contract {
            token,
            token_id,
            owner,
            transactions,
            ..
        } = value;

        let transactions = transactions
            .iter()
            .map(|(key, event)| (*key, event.clone()))
            .collect();
        Self {
            token: token.into(),
            token_id: *token_id,
            owner: *owner,
            transactions,
        }
    }
}

impl From<&Contract> for State {
    fn from(value: &Contract) -> Self {
        let Contract {
            token,
            token_id,
            owner,
            transactions,
            collection,
            constraints,
            referral_metadata,
        } = value;

        let owners = token
            .owner_by_id
            .iter()
            .map(|(hash, actor_id)| (*actor_id, *hash))
            .collect();

        let transactions = transactions
            .iter()
            .map(|(hash, event)| (*hash, event.clone()))
            .collect();

        let token_metadata_by_id = token
            .token_metadata_by_id
            .iter()
            .map(|(id, metadata)| {
                let metadata = metadata.as_ref().unwrap();
                let nft = Nft {
                    owner: token.owner_by_id[id],
                    name: metadata.name.clone(),
                    description: metadata.description.clone(),
                    media_url: metadata.media.clone(),
                    attrib_url: metadata.reference.clone(),
                };
                (*id, nft)
            })
            .collect();

        Self {
            tokens: token_metadata_by_id,
            collection: collection.clone(),
            nonce: *token_id,
            owners,
            owner: *owner,
            transactions,
            constraints: constraints.clone(),
        }
    }
}

fn random() -> u64 {
    use rand_chacha::ChaCha20Rng;
    use rand_core::{RngCore, SeedableRng};

    let subject: [u8; 32] = core::array::from_fn(|i| i as u8 + 1);
    let (seed, block_number) = exec::random(subject).expect("Error in random");

    let mut rng = ChaCha20Rng::from_seed(seed);

    rng.next_u64()
}
