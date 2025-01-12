use hdk::prelude::*;
use fractal_tribute_integrity::*;
use crate::{evm_key_binding::get_evm_address, dna_properties::get_dna_properties};

#[hdk_extern]
pub fn create_game_move(game_move_bytes: Vec<u8>) -> ExternResult<Record> {

    let game_end_time = get_dna_properties(())?.game_end_time;
    let now = sys_time()?.as_seconds_and_nanos().0;
    if now > game_end_time.into() {
        return Err(wasm_error!("Game has ended"));
    }

    let game_move_bytes_slice = game_move_bytes.as_slice();

    let game_move = match GameMove::from_bytes(game_move_bytes_slice) {
        Ok(game_move) => game_move,
        Err(e) => {
            return Err(wasm_error!(e.to_string()));
        }
    };

    if game_move.count_changes() == 0 {
        return Err(wasm_error!("Must change at least one pixel"));
    }
    
    if game_move.count_changes() > 20 {
        return Err(wasm_error!("Max 20 changes allowed"));
    }
    
    let game_move_hash = create_entry(&EntryTypes::GameMove(game_move.clone()))?;
    let _record = get(game_move_hash.clone(), GetOptions::default())?
        .ok_or(
            wasm_error!(
                WasmErrorInner::Guest(String::from("Could not find the newly created GameMove"))
            ),
        )?;

    let path = Path::from("all_game_moves");
    create_link(path.path_entry_hash()?, game_move_hash.clone(), LinkTypes::AllGameMoves, game_move_bytes)?;

    Ok(_record)
}

#[hdk_extern]
pub fn create_tokenid_for_game_move(game_move_hash: ActionHash) -> ExternResult<()> {
    // add the extra 12 empty bytes so it matches the Solidty uint256
    let key_bytes = get_evm_address(())?;

    let content_bytes = game_move_hash.clone().get_raw_39().to_vec();

    let link_base = create_link_base(key_bytes.clone(), content_bytes.clone())?;

    // create the link from the hashed key + content hash to the game_move
    create_link(
        link_base,
        game_move_hash.clone(),
        LinkTypes::TokenIdToGameMove,
        (),
    )?; 
       
    Ok(())
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