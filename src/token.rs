use crate::util;
use hmcdk::api;
use hmcdk::error;
use hmcdk::prelude::*;

pub fn set_approved(owner: &Address, operator: &Address, approved: bool) {
    let key = make_operator_approvals_key(&owner, &operator);
    api::write_state(&key, &approved.to_bytes());
}

pub fn is_approved_for_all(owner: &Address, operator: &Address) -> Result<bool, Error> {
    let key = make_operator_approvals_key(owner, operator);
    api::read_state(&key)
}

fn make_operator_approvals_key(owner: &Address, operator: &Address) -> Vec<u8> {
    util::make_key_by_parts(vec![
        b"operatorApprovals",
        &owner.to_bytes(),
        &operator.to_bytes(),
    ])
}

pub fn set_token_owner(token_id: u64, to: &Address) {
    api::write_state(&make_token_owner_key(token_id), to);
}

pub fn get_token_owner(token_id: u64) -> Result<Address, Error> {
    let key = make_token_owner_key(token_id);
    api::read_state(&key)
}

fn make_token_owner_key(token_id: u64) -> Vec<u8> {
    util::make_key_by_parts(vec![b"tokenOwner", &token_id.to_bytes()])
}

pub fn set_token_approvals(token_id: u64, to: &Address) {
    let key = make_token_approvals_key(token_id);
    api::write_state(&key, &to.to_bytes());
}

fn make_token_approvals_key(token_id: u64) -> Vec<u8> {
    util::make_key_by_parts(vec![b"tokenApprovals", &token_id.to_bytes()])
}

pub fn _get_approved(token_id: u64) -> Result<Vec<u8>, Error> {
    if !check_exists(token_id) {
        return Err(error::from_str(
            "ERC721: approved query for nonexistent token",
        ));
    }
    let key = make_token_approvals_key(token_id);
    api::read_state(&key)
}

pub fn check_exists(token_id: u64) -> bool {
    get_token_owner(token_id).is_ok()
}

pub fn is_approved_or_owner(spender: &Address, token_id: u64) -> Result<bool, Error> {
    if !check_exists(token_id) {
        Err(error::from_str(
            "ERC721: operator query for nonexistent token",
        ))
    } else {
        let owner = get_token_owner(token_id)?;
        Ok(&owner == spender
            || _get_approved(token_id)? == spender
            || is_approved_for_all(&owner, spender)?)
    }
}

pub fn clear_approval(token_id: u64) -> bool {
    match _get_approved(token_id) {
        Ok(_) => {
            let zero_address: [u8; 20] = Default::default();
            set_token_approvals(token_id, &zero_address);
            true
        }
        Err(_) => false,
    }
}
