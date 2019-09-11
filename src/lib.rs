extern crate hmcdk;
use hmcdk::api;
use hmcdk::error;
use hmcdk::prelude::*;
mod mint;
mod token;
mod util;

#[contract]
pub fn init() -> R<bool> {
    mint::set_minter(&api::get_sender()?);
    Ok(Some(true))
}

#[contract]
pub fn approve() -> R<i32> {
    let sender = api::get_sender()?;
    let to: Address = api::get_arg(0)?;
    let token_id: u64 = api::get_arg(1)?;

    let owner = token::get_token_owner(token_id)?;
    if owner == to {
        return Err(error::from_str("ERC721: approval to current owner"));
    }
    if !(owner == sender || token::is_approved_for_all(&owner, &sender)?) {
        return Err(error::from_str(
            "ERC721: approve caller is not owner nor approved for all",
        ));
    }

    token::set_token_approvals(token_id, &to);

    Ok(None)
}

#[allow(non_snake_case)]
#[contract]
pub fn setApprovalForAll() -> R<bool> {
    let sender = api::get_sender()?;
    let to: Address = api::get_arg(0)?;
    let approved: bool = api::get_arg(1)?;

    token::set_approved(&sender, &to, approved);
    // emit ApprovalForAll(msg.sender, to, approved);
    Ok(Some(true))
}

#[allow(non_snake_case)]
#[contract]
pub fn isApprovedForAll() -> R<bool> {
    let owner: Address = api::get_arg(0)?;
    let operator = api::get_arg(1)?;

    if token::is_approved_for_all(&owner, &operator)? {
        Ok(Some(true))
    } else {
        Ok(Some(false))
    }
}

#[allow(non_snake_case)]
#[contract]
pub fn ownerOf() -> R<Address> {
    let token_id: u64 = api::get_arg(0)?;
    Ok(Some(token::get_token_owner(token_id)?))
}

#[contract]
pub fn mint() -> R<u64> {
    let to: Address = api::get_arg(0)?;

    if !mint::is_minter(&api::get_sender()?)? {
        Err(error::from_str("you are not minter"))
    } else {
        let token_id = mint::get_and_incr_next_token_id();
        token::set_token_owner(token_id, &to);
        // FIXME
        // _ownedTokensCount[to].increment();
        Ok(Some(token_id))
    }
}

#[allow(non_snake_case)]
#[contract]
pub fn transferFrom() -> R<i32> {
    let sender: Address = api::get_sender()?;
    let from: Address = api::get_arg(0)?;
    let to = api::get_arg(1)?;
    let token_id: u64 = api::get_arg(2)?;

    if !token::is_approved_or_owner(&sender, token_id)? {
        return Err(error::from_str(
            "ERC721: transfer caller is not owner nor approved",
        ));
    }

    if token::get_token_owner(token_id)? != from {
        return Err(error::from_str("ERC721: transfer of token that is not own"));
    }

    token::clear_approval(token_id);

    // FIXME
    // _ownedTokensCount[from].decrement();
    // _ownedTokensCount[to].increment();

    token::set_token_owner(token_id, &to);

    // FIXME
    // emit Transfer(from, to, tokenId)
    Ok(None)
}

// This API design is a copy from token implementation of crypto kitties: https://etherscan.io/address/0x06012c8cf97bead5deae237070f9587f8e7a266d#code
// WARNING: This method MUST NEVER be called by smart contract code.
#[allow(non_snake_case)]
#[contract]
pub fn tokensOfOwner() -> R<Vec<u8>> {
    let owner: Address = api::get_arg(0)?;

    let current = mint::get_current_token_id();
    if current == 0 {
        return Err(error::from_str("this contract has no tokens"));
    }

    let mut addrs_bytes = Vec::<u8>::new();
    for id in 1..=current {
        if owner == token::get_token_owner(id)? {
            addrs_bytes.extend_from_slice(&id.to_bytes());
        }
    }

    Ok(Some(addrs_bytes))
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
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || {
                    assert_eq!(Some(1u64), mint()?);
                    Ok(())
                })
                .is_ok());
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER2);
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER2, args, || { mint() }).is_err());
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || {
                    let tks = tokensOfOwner()?.unwrap();
                    assert_eq!(1 * 8, tks.len());
                    assert_eq!(1, u64::from_bytes(tks)?);
                    Ok(())
                })
                .unwrap();
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || {
                    assert_eq!(Some(2u64), mint()?);
                    Ok(())
                })
                .is_ok());
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER2);
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || {
                    assert_eq!(Some(3u64), mint()?);
                    Ok(())
                })
                .is_ok());
            }

            {
                let args = {
                    let mut args = ArgsBuilder::new();
                    args.push(SENDER1);
                    args.convert_to_vec()
                };
                hmemu::call_contract(&SENDER1, args, || {
                    let tks = tokensOfOwner()?.unwrap();
                    assert_eq!(2 * 8, tks.len());
                    assert_eq!(1, u64::from_bytes(tks[0..8].to_vec())?);
                    assert_eq!(2, u64::from_bytes(tks[8..16].to_vec())?);
                    Ok(())
                })
                .unwrap();
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
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || {
                    assert_eq!(Some(1u64), mint()?);
                    Ok(())
                })
                .is_ok());
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
                    args.convert_to_vec()
                };
                assert!(hmemu::call_contract(&SENDER1, args, || {
                    assert_eq!(Some(1u64), mint()?);
                    Ok(())
                })
                .is_ok());
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
