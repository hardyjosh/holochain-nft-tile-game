use hdk::prelude::*;
use fractal_tribute_integrity::*;
use crate::evm_key_binding::_get_evm_address;

#[hdk_extern]
pub fn create_game_move(game_move_bytes: Vec<u8>) -> ExternResult<Record> {

    let game_move_bytes_slice: &[u8; 40] = game_move_bytes.as_slice().try_into().map_err(|_| wasm_error!(
        WasmErrorInner::Guest(String::from("Expected a slice of length 40"))
    ))?;  

    let game_move = GameMove::from_bytes(game_move_bytes_slice);
    
    let game_move_hash = create_entry(&EntryTypes::GameMove(game_move.clone()))?;
    let _record = get(game_move_hash.clone(), GetOptions::default())?
        .ok_or(
            wasm_error!(
                WasmErrorInner::Guest(String::from("Could not find the newly created GameMove"))
            ),
        )?;

    // add the extra 12 empty bytes so it matches the Solidty uint256
    let key_result = _get_evm_address();
    let key_bytes = match key_result {
        Ok(key) => {
            match key {
                Some(key) => 
                    key.into_vec()
                ,
                None => {
                    return Err(wasm_error!("No EVM key found"));
                }
            }
        },
        Err(e) => {
            return Err(wasm_error!(e.to_string()));
        }
    };

    let content_bytes = game_move_hash.clone().get_raw_39().to_vec();

    let link_base = create_link_base(key_bytes.clone(), content_bytes.clone())?;

    // create the link from the hashed key + content hash to the game_move
    create_link(
        link_base,
        game_move_hash.clone(),
        LinkTypes::TokenIdToGameMove,
        (),
    )?;

    let path = Path::from("all_game_moves");
    create_link(path.path_entry_hash()?, game_move_hash.clone(), LinkTypes::AllGameMoves, ())?;

    Ok(_record)
}

#[hdk_extern]
pub fn get_game_move(game_move_hash: ActionHash) -> ExternResult<Option<Record>> {
    get(game_move_hash, GetOptions::default())
}

#[hdk_extern]
pub fn get_game_move_from_link(base: ExternalHash) -> ExternResult<Vec<Record>> {
    let links = get_links(base, LinkTypes::TokenIdToGameMove, None)?;
        let get_input: Vec<GetInput> = links
        .into_iter()
        .map(|link| GetInput::new(
            ActionHash::from(link.target).into(),
            GetOptions::default(),
        ))
        .collect();
    let records: Vec<Record> = HDK
        .with(|hdk| hdk.borrow().get(get_input))?
        .into_iter()
        .filter_map(|r| r)
        .collect();
    Ok(records)
}

#[hdk_extern]
pub fn extern_create_link_base(input: LinkBaseInput) -> ExternResult<ExternalHash> {
    fractal_tribute_integrity::create_link_base(input.evm_key, input.content_bytes)
}

#[hdk_extern]
pub fn hash(hash: Vec<u8>) -> ExternResult<Vec<u8>> {
    Ok(hash_keccak256(hash).unwrap())
}