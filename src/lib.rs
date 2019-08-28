extern crate hmc;

static TRUE: &'static [u8] = &[1];
static FALSE: &'static [u8] = &[0];

#[cfg_attr(not(feature = "emulation"), no_mangle)]
pub fn init() -> i32 {
    set_minter(&hmc::get_sender().unwrap());
    0
}

#[cfg_attr(not(feature = "emulation"), no_mangle)]
pub fn approve() -> i32 {
    match _approve() {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(e) => {
            hmc::revert(e);
            -1
        }
    }
}

#[cfg_attr(not(feature = "emulation"), no_mangle)]
#[allow(non_snake_case)]
pub fn setApprovalForAll() -> i32 {
    match _setApprovalForAll() {
        Ok(_) => 0,
        Err(e) => {
            hmc::revert(e);
            -1
        }
    }
}

#[allow(non_snake_case)]
fn _setApprovalForAll() -> Result<(), String> {
    let sender = hmc::get_sender()?;
    let to = hmc::hex_to_bytes(hmc::get_arg_str(0)?.as_ref());
    let approved = hmc::get_arg(1)?;

    if approved != TRUE && approved != FALSE {
        return Err("approved must be true or false".to_string());
    }

    let key = make_operator_approvals_key(&sender, &to);
    hmc::write_state(&key, &approved);
    // emit ApprovalForAll(msg.sender, to, approved);
    Ok(())
}

#[cfg_attr(not(feature = "emulation"), no_mangle)]
#[allow(non_snake_case)]
pub fn isApprovedForAll() -> i32 {
    match _isApprovedForAll() {
        Ok(_) => 0,
        Err(e) => {
            hmc::revert(e);
            -1
        }
    }
}

#[inline(always)]
#[allow(non_snake_case)]
fn _isApprovedForAll() -> Result<i32, String> {
    let owner = hmc::hex_to_bytes(hmc::get_arg_str(0)?.as_ref());
    let operator = hmc::hex_to_bytes(hmc::get_arg_str(1)?.as_ref());

    if is_approved_for_all(&owner, &operator) {
        Ok(hmc::return_value(TRUE))
    } else {
        Ok(hmc::return_value(FALSE))
    }
}

fn is_approved_for_all(owner: &[u8], operator: &[u8]) -> bool {
    let key = make_operator_approvals_key(&owner, &operator);

    match hmc::read_state(&key) {
        Ok(ref v) if v == &TRUE => true,
        Ok(ref v) if v == &FALSE => false,
        _ => false,
    }
}

fn make_operator_approvals_key(owner: &[u8], operator: &[u8]) -> Vec<u8> {
    make_key_by_parts(vec!["operatorApprovals".as_bytes(), &owner, &operator])
}

fn _approve() -> Result<bool, String> {
    let sender = hmc::get_sender()?;
    let to = hmc::hex_to_bytes(hmc::get_arg_str(0)?.as_ref());
    let token_id = hmc::get_arg_str(1)?.parse::<u64>().unwrap();

    let owner = get_token_owner(token_id)?;
    if owner == to {
        return Err("ERC721: approval to current owner".to_string());
    }
    if !(owner == sender || is_approved_for_all(&owner, &sender)) {
        return Err("ERC721: approve caller is not owner nor approved for all".to_string());
    }

    set_token_approvals(token_id, &to);

    Ok(true)
}

fn get_token_owner(token_id: u64) -> Result<Vec<u8>, String> {
    let key = make_token_owner_key(token_id);
    hmc::read_state(&key)
}

#[cfg_attr(not(feature = "emulation"), no_mangle)]
#[allow(non_snake_case)]
pub fn ownerOf() -> i32 {
    match _owner_of() {
        Ok(_) => 0,
        Err(e) => {
            hmc::revert(e);
            -1
        }
    }
}

fn _owner_of() -> Result<(), String> {
    let token_id = hmc::get_arg_str(0)?.parse::<u64>().unwrap();
    let owner = get_token_owner(token_id)?;
    hmc::return_value(&owner);
    Ok(())
}

fn make_token_owner_key(token_id: u64) -> Vec<u8> {
    make_key_by_parts(vec!["tokenOwner".as_bytes(), &token_id.to_be_bytes()])
}

fn set_token_approvals(token_id: u64, to: &[u8]) {
    let key = make_token_approvals_key(token_id);
    hmc::write_state(&key, &to);
}

fn make_token_approvals_key(token_id: u64) -> Vec<u8> {
    make_key_by_parts(vec!["tokenApprovals".as_bytes(), &token_id.to_be_bytes()])
}

fn _get_approved(token_id: u64) -> Result<Vec<u8>, String> {
    if !_exists(token_id) {
        return Err("ERC721: approved query for nonexistent token".to_string());
    }
    let key = make_token_approvals_key(token_id);
    hmc::read_state(&key)
}

fn _exists(token_id: u64) -> bool {
    match get_token_owner(token_id) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn is_approved_or_owner(spender: &[u8], token_id: u64) -> Result<bool, String> {
    if !_exists(token_id) {
        Err("ERC721: operator query for nonexistent token".to_string())
    } else {
        let owner = get_token_owner(token_id)?;
        Ok(owner == spender
            || _get_approved(token_id)? == spender
            || is_approved_for_all(&owner, spender))
    }
}

fn make_key_by_parts(parts: Vec<&[u8]>) -> Vec<u8> {
    parts.join(&('/' as u8))
}

#[cfg_attr(not(feature = "emulation"), no_mangle)]
pub fn mint() -> i32 {
    match _mint() {
        Ok(_) => 0,
        Err(e) => {
            hmc::revert(e);
            -1
        }
    }
}

fn _mint() -> Result<(), String> {
    let to = hmc::hex_to_bytes(hmc::get_arg_str(0)?.as_ref());
    let token_id = hmc::get_arg_str(1)?.parse::<u64>().unwrap();

    if !is_minter(&hmc::get_sender()?) {
        Err("you are not minter".to_string())
    } else if _exists(token_id) {
        Err("token_id already minted".to_string())
    } else {
        set_token_owner(token_id, &to);
        // FIXME
        // _ownedTokensCount[to].increment();
        Ok(())
    }
}

fn set_token_owner(token_id: u64, to: &[u8]) {
    hmc::write_state(&make_token_owner_key(token_id), &to);
}

fn make_minter_key() -> Vec<u8> {
    make_key_by_parts(vec!["minter".as_bytes()])
}

fn set_minter(addr: &[u8]) {
    hmc::write_state(&make_minter_key(), &addr);
}

fn get_minter() -> Vec<u8> {
    hmc::read_state(&make_minter_key()).unwrap()
}

fn is_minter(addr: &[u8]) -> bool {
    get_minter() == addr
}

#[cfg_attr(not(feature = "emulation"), no_mangle)]
#[allow(non_snake_case)]
pub fn transferFrom() -> i32 {
    match _transferFrom() {
        Ok(_) => 0,
        Err(e) => {
            hmc::revert(e);
            -1
        }
    }
}

#[allow(non_snake_case)]
fn _transferFrom() -> Result<(), String> {
    let sender = hmc::get_sender()?;
    let from = hmc::hex_to_bytes(hmc::get_arg_str(0)?.as_ref());
    let to = hmc::hex_to_bytes(hmc::get_arg_str(1)?.as_ref());
    let token_id = hmc::get_arg_str(2)?.parse::<u64>().unwrap();

    if !is_approved_or_owner(&sender, token_id)? {
        return Err("ERC721: transfer caller is not owner nor approved".to_string());
    }

    if get_token_owner(token_id)? != from {
        return Err("ERC721: transfer of token that is not own".to_string());
    }

    clear_approval(token_id);

    // FIXME
    // _ownedTokensCount[from].decrement();
    // _ownedTokensCount[to].increment();

    set_token_owner(token_id, &to);

    // FIXME
    // emit Transfer(from, to, tokenId)
    Ok(())
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

    const SENDER1_ADDR: &str = "0x1221a0726d56aEdeA9dBe2522DdAE3Dd8ED0f36c";
    const SENDER2_ADDR: &str = "0xD8eba1f372b9e0D378259F150d52C2e6C2e4109a";

    #[test]
    fn test_mint() {
        let sender1 = hmc::hex_to_bytes(SENDER1_ADDR);
        let sender2 = hmc::hex_to_bytes(SENDER2_ADDR);

        hmemu::run_process(|| {
            hmemu::call_contract(&sender1, Vec::<String>::new(), || Ok(init())).unwrap();

            assert!(
                hmemu::call_contract(&sender1, vec![SENDER1_ADDR, "1"], || { _mint() }).is_ok()
            );

            assert!(
                hmemu::call_contract(&sender2, vec![SENDER2_ADDR, "2"], || { _mint() }).is_err()
            );

            Ok(0)
        })
        .unwrap();
    }

    #[test]
    fn test_approve() {
        let sender1 = hmc::hex_to_bytes(SENDER1_ADDR);

        hmemu::run_process(|| {
            hmemu::call_contract(&sender1, Vec::<String>::new(), || Ok(init())).unwrap();

            assert!(
                hmemu::call_contract(&sender1, vec![SENDER1_ADDR, "1"], || { _mint() }).is_ok()
            );

            assert!(hmemu::call_contract(&sender1, vec![SENDER2_ADDR, "1"], || {
                _approve()?;
                Ok(0)
            })
            .is_ok());

            Ok(0)
        })
        .unwrap();
    }

    #[test]
    fn test_transfer_from() {
        let sender1 = hmc::hex_to_bytes(SENDER1_ADDR);
        let sender2 = hmc::hex_to_bytes(SENDER2_ADDR);

        hmemu::run_process(|| {
            hmemu::call_contract(&sender1, Vec::<String>::new(), || Ok(init())).unwrap();

            hmemu::call_contract(&sender1, vec![SENDER1_ADDR, "1"], || _mint()).unwrap();

            hmemu::call_contract(&sender1, vec!["1"], || {
                _owner_of()?;
                let owner = hmemu::get_return_value()?;
                assert_eq!(sender1, owner);
                Ok(0)
            })
            .unwrap();

            hmemu::call_contract(&sender2, vec![SENDER1_ADDR, SENDER2_ADDR, "1"], || {
                assert!(_transferFrom().is_err());
                Ok(0)
            })
            .unwrap();

            hmemu::call_contract(&sender1, vec![SENDER1_ADDR, SENDER2_ADDR, "1"], || {
                _transferFrom()
            })
            .unwrap();

            hmemu::call_contract(&sender1, vec!["1"], || {
                _owner_of()?;
                let owner = hmemu::get_return_value()?;
                assert_eq!(sender2, owner);
                Ok(0)
            })
            .unwrap();

            Ok(0)
        })
        .unwrap();

        hmemu::run_process(|| {
            hmemu::call_contract(&sender1, Vec::<String>::new(), || Ok(init())).unwrap();

            hmemu::call_contract(&sender1, vec![SENDER1_ADDR, "1"], || _mint()).unwrap();

            hmemu::call_contract(&sender1, vec![SENDER2_ADDR, "1"], || {
                let ok = _approve()?;
                if ok {
                    Ok(())
                } else {
                    Err("failed to approve".to_string())
                }
            })
            .unwrap();

            hmemu::call_contract(&sender2, vec![SENDER1_ADDR, SENDER2_ADDR, "1"], || {
                _transferFrom()
            })
            .unwrap();

            hmemu::call_contract(&sender1, vec!["1"], || {
                _owner_of()?;
                let owner = hmemu::get_return_value()?;
                assert_eq!(sender2, owner);
                Ok(0)
            })
            .unwrap();

            Ok(0)
        })
        .unwrap();
    }
}
