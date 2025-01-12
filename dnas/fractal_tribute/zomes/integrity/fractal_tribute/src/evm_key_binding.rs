use hdi::prelude::*;
use ethers_core::types::*;

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct EvmKeyBinding {
    pub evm_key: Vec<u8>,
    pub signature_bytes: Vec<u8>,
}

pub fn validate_create_evm_key_binding(
    _action: EntryCreationAction,
    _evm_key_binding: EvmKeyBinding,
) -> ExternResult<ValidateCallbackResult> {

    // first verify the signature
    let mut address_array = [0u8; 20];
    address_array.copy_from_slice(_evm_key_binding.evm_key.as_slice());
    let address = H160::from(address_array);
    let signature: ethers_core::types::Signature = _evm_key_binding.signature_bytes.as_slice().try_into().unwrap();

    let message: RecoveryMessage = _action.author().get_raw_39().try_into().ok().unwrap();

    let verified = signature.verify(message, address);

    if !verified.is_ok() {
        return Ok(
            ValidateCallbackResult::Invalid(
                String::from("EVM pubkey binding signature is invalid"),
            ),
        )
    }

    Ok(ValidateCallbackResult::Valid)
}
pub fn validate_update_evm_key_binding(
    _action: Update,
    _evm_key_binding: EvmKeyBinding,
    _original_action: EntryCreationAction,
    _original_evm_key_binding: EvmKeyBinding,
) -> ExternResult<ValidateCallbackResult> {
    Ok(
        ValidateCallbackResult::Invalid(
            String::from("Evm Key Bindings cannot be updated"),
        ),
    )
}
pub fn validate_delete_evm_key_binding(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_evm_key_binding: EvmKeyBinding,
) -> ExternResult<ValidateCallbackResult> {
    Ok(
        ValidateCallbackResult::Invalid(
            String::from("Evm Key Bindings cannot be deleted"),
        ),
    )
}

pub fn validate_create_link_agent_to_evm_key_binding(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    // Check the entry type for the given action hash
    let action_hash = ActionHash::from(target_address);
    let record = must_get_valid_record(action_hash)?;
    let _evm_key_binding: crate::EvmKeyBinding = record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(
            wasm_error!(
                WasmErrorInner::Guest(String::from("Linked action must reference an entry"))
            ),
        )?;
    // TODO: add the appropriate validation rules
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_delete_link_agent_to_evm_key_binding(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from("All game moves links cannot be deleted")))
}
// #[cfg(test)]
// pub mod tests {
//     use super::*;
//     use holochain::sweettest::*;
//     use futures::future;

//     const DNA_FILEPATH: &str = "../../../workdir/nft_payload.dna";

//     pub mod evm_key_binding_tests {
//         use super::*;
//         use ethers_core::rand::thread_rng;
//         use ethers_signers::{LocalWallet, Signer};

//         #[tokio::test(flavor = "multi_thread")]
//         async fn test_create_evm_binding() {
//             let (conductors, _agents, apps) = setup_conductors(2).await;
//             let conductor: &SweetConductor = &conductors[0];
//             let cells = apps.cells_flattened();
//             let alice = &cells[0];

//             let wallet = LocalWallet::new(&mut thread_rng());

//             // The wallet can be used to sign messages
//             let message = alice.agent_pubkey().get_raw_39();
//             let signature = wallet.sign_message(message).await.unwrap();
//             assert_eq!(signature.recover(&message[..]).unwrap(), wallet.address());

//             let evm_key_binding = EvmKeyBinding {
//                 evm_key: ByteArray(wallet.address().as_bytes().to_vec()),
//                 signature_bytes: ByteArray(signature.to_vec()),
//             };

//             let record: Record = conductor
//                 .call(
//                     &alice.zome("nft_payload"), 
//                     "create_evm_key_binding", 
//                     evm_key_binding.clone()
//                 ).await;

//             println!("{:#?}", record);
//         }

//         #[tokio::test(flavor = "multi_thread")]
//         #[should_panic(expected = "EVM pubkey binding signature is invalid")]        
//         async fn test_create_evm_binding_bad_sig() {
//             let (conductors, _agents, apps) = setup_conductors(2).await;
//             let conductor: &SweetConductor = &conductors[0];
//             let cells = apps.cells_flattened();
//             let alice = &cells[0];

//             let wallet = LocalWallet::new(&mut thread_rng());
//             let second_wallet = LocalWallet::new(&mut thread_rng());

//             // The wallet can be used to sign messages
//             let message = alice.agent_pubkey().get_raw_39();
//             let signature = wallet.sign_message(message).await.unwrap();
//             assert_eq!(signature.recover(&message[..]).unwrap(), wallet.address());

//             let evm_key_binding = EvmKeyBinding {
//                 evm_key: ByteArray(second_wallet.address().as_bytes().to_vec()),
//                 signature_bytes: ByteArray(signature.to_vec()),
//             };

//             let record: Record = conductor
//                 .call(
//                     &alice.zome("nft_payload"), 
//                     "create_evm_key_binding", 
//                     evm_key_binding.clone()
//                 ).await;

//             println!("{:#?}", record);
//         }

//     }

//     async fn setup_conductors(n: usize) -> (SweetConductorBatch, Vec<AgentPubKey>, SweetAppBatch) {
//         let dna = SweetDnaFile::from_bundle(std::path::Path::new(DNA_FILEPATH))
//             .await
//             .unwrap();

//         let mut conductors = SweetConductorBatch::from_standard_config(n).await;

//         let all_agents: Vec<AgentPubKey> =
//             future::join_all(conductors.iter().map(|c| SweetAgents::one(c.keystore()))).await;
//         let apps = conductors
//             .setup_app_for_zipped_agents("app", &all_agents, &[dna])
//             .await
//             .unwrap();

//         conductors.exchange_peer_info().await;
//         (conductors, all_agents, apps)
//     }
// }