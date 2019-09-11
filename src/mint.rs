use crate::util;
use hmcdk::api;
use hmcdk::prelude::*;

pub fn make_minter_key() -> Vec<u8> {
    util::make_key_by_parts(vec![b"minter"])
}

pub fn set_minter(addr: &Address) {
    api::write_state(&make_minter_key(), &addr.to_bytes());
}

pub fn get_minter() -> Result<Address, Error> {
    api::read_state(&make_minter_key())
}

pub fn is_minter(addr: &Address) -> Result<bool, Error> {
    Ok(&get_minter()? == addr)
}

fn make_token_seq_key() -> Vec<u8> {
    util::make_key_by_parts(vec![b"mint"])
}

pub fn get_and_incr_next_token_id() -> u64 {
    let id: u64 = get_current_token_id() + 1;
    api::write_state(&make_token_seq_key(), &id.to_bytes());
    id
}

pub fn get_current_token_id() -> u64 {
    let key = make_token_seq_key();
    match api::read_state(&key) {
        Ok(v) => v,
        Err(_) => 0,
    }
}
