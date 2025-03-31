use soroban_sdk::{Env, Address, Val, IntoVal};
use crate::storage_types::DataKey;

pub struct Allowance {
    pub amount: i128,
    pub expiration_ledger: u32,
}

pub fn write_approval(
    e: &Env,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let allowance = Allowance {
        amount,
        expiration_ledger,
    };
    e.storage()
        .instance()
        .set(&DataKey::Allowance(from, spender), &allowance);
}

pub fn read_approval(e: &Env, from: Address, spender: Address) -> Allowance {
    e.storage()
        .instance()
        .get(&DataKey::Allowance(from, spender))
        .unwrap_or(Allowance {
            amount: 0,
            expiration_ledger: 0,
        })
}

pub fn spend_allowance(e: &Env, from: Address, spender: Address, amount: i128) {
    let mut allowance = read_approval(e, from.clone(), spender.clone());
    allowance.amount -= amount;
    write_approval(e, from, spender, allowance.amount, allowance.expiration_ledger);
}