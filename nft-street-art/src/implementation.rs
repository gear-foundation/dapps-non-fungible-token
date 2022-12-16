use crate::{
    City, Country, Metadata, NFTAction, NFTCore, NFTEvent, NFTTransfer, TokenId, H256, NFT, U256,
};
use gstd::{exec, msg, prelude::*, ActorId};

const DELAY: u32 = 600_000;
const INTERVAL: u32 = 2_592_000_000;

#[allow(clippy::too_many_arguments)]
pub trait MyNFTCore: NFTCore {
    fn mint(
        &mut self,
        account: &ActorId,
        name: String,
        collection_name: String,
        description: String,
        social_network: String,
        country_and_city: Option<(Country, City)>,
        link_to_media: Option<String>,
        coordinates: Option<String>,
    ) -> NFTTransfer;
    fn generate_token(&mut self, owner: &ActorId, parent_id: TokenId) -> NFTTransfer;
}

impl MyNFTCore for NFT {
    fn mint(
        &mut self,
        account: &ActorId,
        name: String,
        collection_name: String,
        description: String,
        social_network: String,
        country_and_city: Option<(Country, City)>,
        link_to_media: Option<String>,
        coordinates: Option<String>,
    ) -> NFTTransfer {
        assert_eq!(msg::source(), self.owner, "Only admin can create NFTs");
        let transfer = NFTCore::mint(self, account, self.token_id, None);
        let mut metadata = Metadata {
            name,
            collection_name: Some(collection_name),
            description: Some(description),
            social_network,
            country_and_city,
            link_to_media,
            coordinates,
            ..Default::default()
        };
        let token_id = self.token_id;
        if street_art_is_finished(&metadata) {
            metadata.created_at = Some(exec::block_timestamp());
            msg::send_delayed(
                exec::program_id(),
                NFTAction::GenerateToken {
                    parent_id: token_id,
                    owner: msg::source(),
                },
                0,
                INTERVAL,
            )
            .expect("Error in sending a delayed message `NFTAction::GenerateToken`");
        }
        self.id_to_metadata.insert(token_id, metadata);

        self.token_id = self.token_id.saturating_add(U256::one());
        transfer
    }

    fn generate_token(&mut self, owner: &ActorId, parent_id: TokenId) -> NFTTransfer {
        assert_eq!(
            exec::program_id(),
            msg::source(),
            "Only the contract itself can generate a new token"
        );
        let transfer = NFTCore::mint(self, owner, self.token_id, None);
        let token_id = self.token_id;
        self.token_id = self.token_id.saturating_add(U256::one());

        self.id_to_metadata
            .entry(parent_id)
            .and_modify(|metadata| metadata.child_token_id = Some(token_id));

        let country_and_city = get_country_and_city(&self.countries_to_cities);

        let metadata = Metadata {
            country_and_city: Some(country_and_city),
            ..Default::default()
        };
        self.id_to_metadata.insert(token_id, metadata);

        transfer
    }
}

pub fn street_art_is_finished(metadata: &Metadata) -> bool {
    metadata.country_and_city.is_some()
        && metadata.link_to_media.is_some()
        && metadata.coordinates.is_some()
        && metadata.collection_name.is_some()
        && metadata.description.is_some()
}

static mut SEED: u64 = 0;

fn get_country_and_city(contries_cities: &BTreeMap<Country, BTreeSet<City>>) -> (Country, City) {
    let seed = unsafe {
        SEED = SEED.wrapping_add(1);
        SEED
    };
    let contries_cities_vec = Vec::from_iter(contries_cities.iter());
    let countries_len: u8 = contries_cities_vec.len() as u8;
    let random = exec::random(&(exec::block_timestamp() + seed).to_be_bytes()).expect("");
    let r1: usize = (random.0[0] % countries_len) as usize;
    let country = contries_cities_vec[r1].0.clone();
    let cities: Vec<City> = contries_cities_vec[r1].1.iter().cloned().collect();
    let cities_len: u8 = cities.len() as u8;
    let r2: usize = (random.0[1] % cities_len) as usize;
    let city = cities[r2].clone();
    (country, city)
}

impl NFT {
    pub fn update_token(
        &mut self,
        token_id: TokenId,
        link_to_media: Option<String>,
        coordinates: Option<String>,
        description: Option<String>,
        collection_name: Option<String>,
    ) {
        if let Some(owner) = self.token.owner_by_id.get(&token_id) {
            assert_eq!(
                owner,
                &msg::source(),
                "Only owner can update information about token"
            );
        } else {
            panic!("Token does not exist")
        }
        let metadata = self
            .id_to_metadata
            .get_mut(&token_id)
            .expect("Error in getting token metadata");
        if metadata.created_at.is_some() {
            panic!("That street art has already been finished")
        }
        if link_to_media.is_some() {
            metadata.link_to_media = link_to_media;
        }
        if coordinates.is_some() {
            metadata.coordinates = coordinates;
        }
        if description.is_some() {
            metadata.description = description;
        }
        if collection_name.is_some() {
            metadata.collection_name = collection_name;
        }
        if street_art_is_finished(metadata) {
            metadata.created_at = Some(exec::block_timestamp());
            msg::send_delayed(
                exec::program_id(),
                NFTAction::GenerateToken {
                    parent_id: token_id,
                    owner: msg::source(),
                },
                0,
                INTERVAL,
            )
            .expect("Error in sending a delayed message `NFTAction::GenerateToken`");
        }
        msg::reply(NFTEvent::TokenUpdated, 0)
            .expect("Error in sending a reply `NFTEvent::TokenUpdated`");
    }

    pub fn add_countries_and_cities(&mut self, countries_and_cities: &Vec<(Country, Vec<City>)>) {
        assert_eq!(
            msg::source(),
            self.owner,
            "Only contract owner can modify countries and cities list"
        );
        for (country, cities) in countries_and_cities {
            let mut new_cities: BTreeSet<City> = cities.iter().cloned().collect();
            self.countries_to_cities
                .entry(country.clone())
                .and_modify(|cities| {
                    cities.append(&mut new_cities);
                })
                .or_insert_with(|| new_cities);
        }
        msg::reply(NFTEvent::CountryCitiesAdded, 0)
            .expect("Error in sending a reply `NFTEvent::CountryCitiesAdded`");
    }

    pub fn remove_countries_and_cities(
        &mut self,
        countries_and_cities: &Vec<(Country, Option<Vec<City>>)>,
    ) {
        assert_eq!(
            msg::source(),
            self.owner,
            "Only contract owner can modify countries and cities list"
        );
        for (country, cities) in countries_and_cities {
            if let Some(rm_cities) = cities {
                self.countries_to_cities
                    .entry(country.clone())
                    .and_modify(|cities| {
                        for city in rm_cities {
                            cities.remove(city);
                        }
                    });
            } else {
                self.countries_to_cities.remove(country);
            }
        }
        msg::reply(NFTEvent::CountryCitiesRemoved, 0)
            .expect("Error in sending a reply `NFTEvent::CountryCitiesRemoved`");
    }

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
            "Not allowed to creal transactions"
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
    .expect("Error in sending a delayled message `FTStorageAction::Clear`");
}
