extern crate hmcdk;
use hmcdk::api;
use hmcdk::error;
use hmcdk::prelude::*;

#[contract]
pub fn init() -> R<bool> {
    set_minter(&api::get_sender()?);
    Ok(Some(true))
}

#[contract]
pub fn approve() -> R<i32> {
    let sender = api::get_sender()?;
    let to: Address = api::get_arg(0)?;
    let token_id: u64 = api::get_arg(1)?;

    let owner = get_token_owner(token_id)?;
    if owner == to {
        return Err(error::from_str("ERC721: approval to current owner"));
    }
    if !(owner == sender || is_approved_for_all(&owner, &sender)?) {
        return Err(error::from_str(
            "ERC721: approve caller is not owner nor approved for all",
        ));
    }

    set_token_approvals(token_id, &to);

    Ok(None)
}

#[allow(non_snake_case)]
#[contract]
pub fn setApprovalForAll() -> R<bool> {
    let sender = api::get_sender()?;
    let to: Address = api::get_arg(0)?;
    let approved: bool = api::get_arg(1)?;
    let key = make_operator_approvals_key(&sender, &to);
    api::write_state(&key, &approved.to_bytes());
    // emit ApprovalForAll(msg.sender, to, approved);
    Ok(Some(true))
}

#[allow(non_snake_case)]
#[contract]
pub fn isApprovedForAll() -> R<bool> {
    let owner: Address = api::get_arg(0)?;
    let operator = api::get_arg(1)?;

    if is_approved_for_all(&owner, &operator)? {
        Ok(Some(true))
    } else {
        Ok(Some(false))
    }
}

fn is_approved_for_all(owner: &Address, operator: &Address) -> Result<bool, Error> {
    let key = make_operator_approvals_key(owner, operator);
    api::read_state(&key)
}

fn make_operator_approvals_key(owner: &Address, operator: &Address) -> Vec<u8> {
    make_key_by_parts(vec![
        b"operatorApprovals",
        &owner.to_bytes(),
        &operator.to_bytes(),
    ])
}

fn get_token_owner(token_id: u64) -> Result<Address, Error> {
    let key = make_token_owner_key(token_id);
    api::read_state(&key)
}

#[allow(non_snake_case)]
#[contract]
pub fn ownerOf() -> R<Address> {
    let token_id: u64 = api::get_arg(0)?;
    Ok(Some(get_token_owner(token_id)?))
}

fn make_token_owner_key(token_id: u64) -> Vec<u8> {
    make_key_by_parts(vec![b"tokenOwner", &token_id.to_bytes()])
}

fn set_token_approvals(token_id: u64, to: &Address) {
    let key = make_token_approvals_key(token_id);
    api::write_state(&key, &to.to_bytes());
}

fn make_token_approvals_key(token_id: u64) -> Vec<u8> {
    make_key_by_parts(vec![b"tokenApprovals", &token_id.to_bytes()])
}

fn _get_approved(token_id: u64) -> Result<Vec<u8>, Error> {
    if !_exists(token_id) {
        return Err(error::from_str(
            "ERC721: approved query for nonexistent token",
        ));
    }
    let key = make_token_approvals_key(token_id);
    api::read_state(&key)
}

fn _exists(token_id: u64) -> bool {
    get_token_owner(token_id).is_ok()
}

fn is_approved_or_owner(spender: &Address, token_id: u64) -> Result<bool, Error> {
    if !_exists(token_id) {
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

fn make_key_by_parts(parts: Vec<&[u8]>) -> Vec<u8> {
    parts.join(&b'/')
}

#[contract]
pub fn mint() -> R<i32> {
    let to: Address = api::get_arg(0)?;
    let token_id: u64 = api::get_arg(1)?;

    if !is_minter(&api::get_sender()?)? {
        Err(error::from_str("you are not minter"))
    } else if _exists(token_id) {
        Err(error::from_str("token_id already minted"))
    } else {
        set_token_owner(token_id, &to);
        // FIXME
        // _ownedTokensCount[to].increment();
        Ok(None)
    }
}

fn set_token_owner(token_id: u64, to: &Address) {
    api::write_state(&make_token_owner_key(token_id), to);
}

fn make_minter_key() -> Vec<u8> {
    make_key_by_parts(vec![b"minter"])
}

fn set_minter(addr: &Address) {
    api::write_state(&make_minter_key(), &addr.to_bytes());
}

fn get_minter() -> Result<Address, Error> {
    api::read_state(&make_minter_key())
}

fn is_minter(addr: &Address) -> Result<bool, Error> {
    Ok(&get_minter()? == addr)
}

#[allow(non_snake_case)]
#[contract]
pub fn transferFrom() -> R<i32> {
    let sender: Address = api::get_sender()?;
    let from: Address = api::get_arg(0)?;
    let to = api::get_arg(1)?;
    let token_id: u64 = api::get_arg(2)?;

    if !is_approved_or_owner(&sender, token_id)? {
        return Err(error::from_str(
            "ERC721: transfer caller is not owner nor approved",
        ));
    }

    if get_token_owner(token_id)? != from {
        return Err(error::from_str("ERC721: transfer of token that is not own"));
    }

    clear_approval(token_id);

    // FIXME
    // _ownedTokensCount[from].decrement();
    // _ownedTokensCount[to].increment();

    set_token_owner(token_id, &to);

    // FIXME
    // emit Transfer(from, to, tokenId)
    Ok(None)
}

fn clear_approval(token_id: u64) -> bool {
    match _get_approved(token_id) {
        Ok(_) => {
            let zero_address: [u8; 20] = Default::default();
            set_token_approvals(token_id, &zero_address);
            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    extern crate hmemu;
    use super::*;
    use hmemu::types::ArgsBuilder;

    const SENDER1: Address = *b"00000000000000000001";
    const SENDER2: Address = *b"00000000000000000002";

    #[test]
    fn test_mint() {
        hmemu::run_process(|| {
            let _ = hmemu::call_contract(&SENDER1, vec![], || Ok(init())).unwrap();

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || { mint() }).is_ok());
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER2);
                    args.push(2u64);
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER2, args, || { mint() }).is_err());
            }

            Ok(0)
        })
        .unwrap();
    }

    #[test]
    fn test_approve() {
        hmemu::run_process(|| {
            let _ = hmemu::call_contract(&SENDER1, vec![], || Ok(init())).unwrap();

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || { mint() }).is_ok());
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER2);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || {
                    approve()?;
                    Ok(0)
                })
                .is_ok());
            }

            Ok(0)
        })
        .unwrap();
    }

    #[test]
    fn test_transfer_from() {
        hmemu::run_process(|| {
            let _ = hmemu::call_contract(&SENDER1, vec![], || Ok(init())).unwrap();

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || mint()).unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || {
                    let owner = ownerOf()?;
                    assert_eq!(Some(SENDER1), owner);
                    Ok(0)
                })
                .unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.push(SENDER2);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER2, args, || {
                    assert!(transferFrom().is_err());
                    Ok(0)
                })
                .unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.push(SENDER2);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || transferFrom()).unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || {
                    let owner = ownerOf()?;
                    assert_eq!(Some(SENDER2), owner);
                    Ok(0)
                })
                .unwrap();
            }

            Ok(0)
        })
        .unwrap();

        hmemu::run_process(|| {
            let _ = hmemu::call_contract(&SENDER1, vec![], || Ok(init())).unwrap();

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || mint()).unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER2);
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || {
                    if approve().is_ok() {
                        Ok(())
                    } else {
                        Err(error::from_str("failed to approve"))
                    }
                })
                .unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.push(SENDER2);
                    args.push(1u64);
                    args.convert_to_vec()
                };

                hmemu::call_contract(&SENDER2, args, || transferFrom()).unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(1u64);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || {
                    let owner = ownerOf()?;
                    assert_eq!(Some(SENDER2), owner);
                    Ok(0)
                })
                .unwrap();
            }

            Ok(0)
        })
        .unwrap();
    }
}
