use crate::{ContractId, MarketEvent};
use ft_io::*;
use gstd::{exec, msg, ActorId};

const MINIMUM_VALUE: u64 = 500;

pub async fn transfer_tokens(contract_id: ContractId, from: ActorId, to: ActorId, amount: u128) {
    msg::send_for_reply(contract_id, FTAction::Transfer { from, to, amount }, 0)
        .expect("Error in sending message `FTAction::Transfer` to the fungible token contract")
        .await
        .expect("Error in decoding `FTEvent`");
}

pub async fn transfer_payment(
    from: ActorId,
    to: ActorId,
    ft_contract_id: Option<ContractId>,
    price: u128,
) {
    if let Some(contract_id) = ft_contract_id {
        transfer_tokens(contract_id, from, to, price).await;
    } else if to != exec::program_id() && price > MINIMUM_VALUE.into() {
        msg::send(to, MarketEvent::Payment, price).expect("Error in sending payment in value");
    }
}

pub fn check_attached_value(ft_contract_id: Option<ActorId>, price: u128) {
    if ft_contract_id.is_none() && msg::value() != price {
        panic!("attached value is not equal the indicated price");
    }
}
