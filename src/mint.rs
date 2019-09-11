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
