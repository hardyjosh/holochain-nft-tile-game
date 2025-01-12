use hdk::prelude::*;
use fractal_tribute_integrity::*;
use crate::all_game_moves::*;
use ethers_core::types::U256;

#[hdk_extern]
pub fn get_latest_board(_: ()) -> ExternResult<BoardWithMetadata> {
    let game_moves = get_all_game_moves_from_link_tags(())?;

    let board = Board::reconstruct_from_game_moves(&game_moves);
    let bytes = board.to_bytes();
    let svg = board.generate_svg();
    let complete_svg = board.generate_svg_with_defs();
    let creator = HoloHash::from_raw_36([0; 36].to_vec());
    let creation_hash = HoloHash::from_raw_36([0; 36].to_vec());

    Ok(
        BoardWithMetadata {
            complete_svg,
            svg,
            bytes,
            creator,
            creation_hash,
        }
    )
}

#[hdk_extern]
pub fn get_board_at_move(move_action_hash: ActionHash) -> ExternResult<BoardWithMetadata> {
    let board = _get_board_at_move(move_action_hash.clone())?;
    let bytes = board.to_bytes();
    let svg = board.generate_svg();
    let complete_svg = board.generate_svg_with_defs();
    let creator = must_get_valid_record(move_action_hash.clone())?.action().author().clone();

    Ok(
        BoardWithMetadata {
            complete_svg,
            svg,
            bytes,
            creator,
            creation_hash: move_action_hash,
        }
    )
}

pub fn _get_board_at_move(move_action_hash: ActionHash) -> ExternResult<Board> {
    let path = hdk::hash_path::path::Path::from("all_game_moves");
    let links: Vec<Link> = get_links(path.path_entry_hash()?, LinkTypes::AllGameMoves, None)?;

    // truncate the links
    let mut links = links.clone();
    if let Some(pos) = links.clone().iter().position(|link| link.target.clone().into_action_hash().unwrap().eq(&move_action_hash)) {
        links.truncate(pos + 1)
    } else {
        return Err(wasm_error!("Could not find a game move for that action hash"));
    }

    // get the bytes from each link tag and make a vector of game moves
    let game_moves: Vec<GameMove> = links.clone()
        .into_iter()
        .map(|link| {
            let bytes = link.tag.0;
            let game_move = GameMove::from_bytes(&bytes).ok().unwrap();
            Ok(game_move)
        })
        .collect::<ExternResult<Vec<GameMove>>>()?;

    let board = Board::reconstruct_from_game_moves(&game_moves);

    Ok(board)
}

#[hdk_extern]
pub fn get_boards_from_moves(move_action_hashes: Vec<ActionHash>) -> ExternResult<Vec<BoardWithMetadata>> {
    let mut boards = Vec::new();
    for move_action_hash in move_action_hashes {
        let board = get_board_at_move(move_action_hash)?;
        boards.push(board);
    }
    Ok(boards)
}

#[hdk_extern]
pub fn get_boards_from_all_my_moves(_:()) -> ExternResult<Vec<BoardWithMetadata>> {
    let moves = get_all_my_game_moves(())?;
    let action_hashes = moves.into_iter().map(|record| {
        record.action_hashed().as_hash().clone()
    }).collect::<Vec<ActionHash>>();
    get_boards_from_moves(action_hashes)
}

#[hdk_extern]
pub fn get_board_from_link(base: ExternalHash) -> ExternResult<BoardWithMetadataAndId> {
    let links = get_links(base.clone(), LinkTypes::TokenIdToGameMove, None)?;
    if let 0 = links.len() {
        return Err(wasm_error!("No game moves found for that token id"));
    }
    let action_hash = links[0].target.clone();
    let board_with_creator = get_board_at_move(action_hash.into())?;
    let id = base.get_raw_32().to_vec();
    Ok(BoardWithMetadataAndId {
        board: board_with_creator,
        id,
    })
}

pub fn _get_board_from_link(base: ExternalHash) -> ExternResult<Board> {
    let links = get_links(base.clone(), LinkTypes::TokenIdToGameMove, None)?;
    if let 0 = links.len() {
        return Err(wasm_error!("No game moves found for that token id"));
    }
    let action_hash = links[0].target.clone();
    let board = _get_board_at_move(action_hash.into())?;
    Ok(board)
}

#[hdk_extern]
pub fn get_boards_from_links(link_bases: Vec<ExternalHash>) -> ExternResult<Vec<BoardWithMetadataAndId>> {
    let mut boards = Vec::new();
    for link_base in link_bases {
        if let Ok(board) = get_board_from_link(link_base) {
            boards.push(board);
        }
    }
    Ok(boards)
}

fn token_id_to_board(token_id: U256) -> ExternResult<Board> {
    let mut link_base = [0u8; 39];
    link_base[..3].copy_from_slice(&[132, 47, 36]);
    token_id.to_big_endian(&mut link_base[3..35]);
    link_base[35..].copy_from_slice(&[0u8; 4]);
    debug!("link_base: {:?}", link_base);
    let link_base = HoloHash::from_raw_39(link_base.to_vec()).map_err(|_| wasm_error!("Could not parse token id"))?;
    let board = _get_board_from_link(link_base)?;
    Ok(board)
}

// a function that will create a metadata json for an nft, given a token id
#[hdk_extern]
fn token_id_to_metadata(str: String) -> ExternResult<String> {
    let token_id = U256::from_dec_str(&str).map_err(|_| wasm_error!("Could not parse token id"))?;
    let board = token_id_to_board(token_id)?;
    let svg = board.generate_svg_data_uri();
    let metadata = Metadata {
        name: "Fractal Tribute".to_string(),
        description: "A collaborative art game".to_string(),
        image: svg,
    };
    let json = serde_json::to_string(&metadata).map_err(|_| wasm_error!("Could not serialize metadata"))?;
    Ok(json)
}
