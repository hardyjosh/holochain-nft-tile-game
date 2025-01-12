import type { Board, BoardWithMetadataAndId, EvmKeyBinding, GameMove, GameMoveWithActionHash, IncomingBoardWithMetadataAndId, ParticipationProof, BoardWithMetadata, IncomingBoardWithMetadata, DnaProperties, TransformedDnaProperties, Profile, AgentParticipation } from '$lib/types';
import type { AppAgentClient, Record, ActionHash, AgentPubKey } from '@holochain/client';
import { writable } from 'svelte/store';
import { type Address, getAddress, bytesToHex, concat, hexToBytes } from 'viem'
import { decode } from "@msgpack/msgpack";
import { gameMoveToBytes, parseBoardBytes, parseIncomingBoardWithMetadata, parseIncomingBoardWithMetadataAndId, tokenIdToLinkBase, transformParticipationProof } from '$lib/helpers';
import { transformDnaProperties } from '$lib/helpers/dna-properties';
import type WebSdkApi from '@holo-host/web-sdk';
import WebSdk from "@holo-host/web-sdk";
import { AppAgentWebsocket } from "@holochain/client";
import { setIsHotHolder } from '$lib/stores/hotHolder';


export const happ = writable<DnaInterface>()

let client: AppAgentWebsocket | WebSdk;

const IS_HOLO = ["true", "1", "t"].includes(
    import.meta.env.VITE_APP_IS_HOLO?.toLowerCase()
);

export const initHapp = async () => {
    if (IS_HOLO) {
        client = await WebSdk.connect({
            chaperoneUrl: "https://chaperone-demo.holo.hosting",
            authFormCustomization: {
                appName: "svelte-holo-test",
            },
        });
        const waitForAgentState = () => new Promise(resolve => {
            (client as WebSdk).on("agent-state", (agent_state) => {
                console.log('agent state boo', agent_state)
                if (agent_state?.isAvailable) {
                    console.log('should resolve')
                    resolve(null);
                }
            });
        });

        // (client as WebSdk).signUp({ cancellable: false });

        await waitForAgentState();
    } else {
        client = await AppAgentWebsocket.connect("", "svelte-holo-test");
    }

    const iface = new DnaInterface(client);
    await iface.init();
    happ.set(iface);
};

const role_name = 'fractal_tribute';
const zome_name = 'fractal_tribute';

export class DnaInterface {

    constructor(client: AppAgentClient | WebSdkApi) {
        this.client = client;
    }

    public client: AppAgentClient | WebSdkApi;
    public dnaProperties: TransformedDnaProperties;

    myPubKey(): Uint8Array {
        return this.client.myPubKey;
    }

    async init() {
        this.dnaProperties = await this.getDnaInfo()
        console.log(this.dnaProperties)
    }

    async getDnaInfo(): Promise<TransformedDnaProperties> {
        try {
            const res = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_dna_properties',
                payload: null,
            }) as DnaProperties
            // console.log(res)
            return transformDnaProperties(res)
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
        }
    }

    // evm key binding
    async createEvmKeyBinding(evmKeyBindingEntry: EvmKeyBinding): Promise<Record> {
        let _evmKeyBinding: any = {}
        _evmKeyBinding.evm_key = Array.from(evmKeyBindingEntry.evm_key)
        _evmKeyBinding.signature_bytes = Array.from(evmKeyBindingEntry.signature_bytes)
        try {
            const record = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'create_evm_key_binding',
                payload: _evmKeyBinding,
            }) as Record
            setIsHotHolder(bytesToHex(evmKeyBindingEntry.evm_key));
            return record;
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
        }
    }

    async getEvmAddress(): Promise<Address> {
        try {
            let addressBytes = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_evm_address',
                payload: null,
            })
            if (!addressBytes) {
                return null
            }
            return getAddress(bytesToHex(addressBytes))
        } catch (e) {
            if (e?.message.toString().includes('No EvmKeyBinding found for this agent')) return null
            console.log(e?.data?.data || e)
            // console.log(e?.message.toString().includes('Record not found'))
        }
    }

    private agentEvmKeys: Map<string, Address> = new Map();

    async getAgentEvmAddress(agent: AgentPubKey): Promise<Address> {
        // if we already have the address, return it
        if (this.agentEvmKeys.has(agent.toString())) {
            return this.agentEvmKeys.get(agent.toString());
        }
        try {
            let addressBytes = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_agent_evm_address',
                payload: agent,
            })
            if (!addressBytes) {
                return null
            }
            const address = getAddress(bytesToHex(addressBytes))
            this.agentEvmKeys.set(agent.toString(), address);
            return address
        } catch (e) {
            console.log(e?.data?.data || e)
            // console.log(e?.message.toString().includes('Record not found'))
        }
    }

    // profile
    async createProfile(profile: Profile): Promise<Record> {
        try {
            return await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'create_profile',
                payload: profile,
            }) as Record
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
        }
    }

    async getProfile(agentPubKey: Uint8Array): Promise<Profile> {
        try {
            const profile = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_profile',
                payload: agentPubKey,
            }) as Profile
            return profile
        } catch (e) {
            if (e?.message.toString().includes('No profile found for this agent')) return null
            console.log(e?.data?.data)
            console.log(e)
        }
    }


    // moves
    async createGameMove(gameMove: GameMove): Promise<Record> {
        // console.log('creating move', gameMove)
        try {
            return await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'create_game_move',
                payload: Array.from(gameMoveToBytes(gameMove)),
            }) as Record
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
            throw (e)
        }
    }

    async getAllMyGameMoves(): Promise<GameMoveWithActionHash[]> {
        try {
            const request = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_all_my_game_moves',
                payload: null,
            })
            const records: GameMoveWithActionHash[] = request.map((r: Record) => {
                const gameMove = decode((r.entry as any).Present.entry) as GameMove
                const actionHash = r.signed_action.hashed.hash
                return { gameMove, actionHash }
            })
            // console.log('all my game moves', records)
            return records
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)

        }
    }

    async createTokenIdForGameMove(gameMove: ActionHash) {
        try {
            return await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'create_tokenid_for_game_move',
                payload: gameMove,
            })
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
            throw (e)
        }
    }

    async getNumberOfMoves(): Promise<number> {
        try {
            return await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_number_of_moves',
                payload: null,
            })
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
        }
    }

    // create_favourite_move
    async createFavouriteMove(gameMoveHash: ActionHash): Promise<void> {
        try {
            await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'create_favourite_move',
                payload: gameMoveHash,
            });
        } catch (e) {
            console.log(e?.data?.data || e);
        }
    }

    // get_favourite_moves_for_agent
    async getFavouriteMovesForAgent(agentPubkey: AgentPubKey): Promise<GameMoveWithActionHash[]> {
        try {
            const request = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_favourite_moves_for_agent',
                payload: agentPubkey,
            }) as Record[];
            const records: GameMoveWithActionHash[] = request.map((r: Record) => {
                const gameMove = decode((r.entry as any).Present.entry) as GameMove
                const actionHash = r.signed_action.hashed.hash
                return { gameMove, actionHash }
            })
            return records
        } catch (e) {
            console.log(e?.data?.data || e);
            return [];
        }
    }

    // get_favourite_moves_for_current_agent
    async getFavouriteMovesForCurrentAgent(): Promise<GameMoveWithActionHash[]> {
        try {
            const request = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_favourite_moves_for_current_agent',
                payload: null,
            }) as Record[];
            const records: GameMoveWithActionHash[] = request.map((r: Record) => {
                const gameMove = decode((r.entry as any).Present.entry) as GameMove
                const actionHash = r.signed_action.hashed.hash
                return { gameMove, actionHash }
            })
            return records
        } catch (e) {
            console.log(e?.data?.data || e);
            return [];
        }
    }

    // board
    async getLatestBoard(): Promise<BoardWithMetadata> {
        try {
            const res = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_latest_board',
                payload: null,
            }) as IncomingBoardWithMetadata
            return parseIncomingBoardWithMetadata(res)
        } catch (e) {
            console.log("Error getting latest board", e?.data?.data || e)
        }
    }

    async getBoardAtMove(actionHash: ActionHash): Promise<BoardWithMetadata> {
        try {
            const board = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_board_at_move',
                payload: actionHash,
            }) as IncomingBoardWithMetadata
            return parseIncomingBoardWithMetadata(board);
        } catch (e) {
            console.log("Error getting board at move", e?.data?.data || e)
        }

    }

    async getBoardFromTokenId(tokenId: Uint8Array): Promise<BoardWithMetadataAndId> {
        try {
            const linkBase = tokenIdToLinkBase(tokenId);
            const incomingBoard = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_board_from_link',
                payload: linkBase,
            }) as IncomingBoardWithMetadataAndId

            return parseIncomingBoardWithMetadataAndId(incomingBoard)
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
        }
    }

    async getBoardsFromTokenIds(tokenIds: Uint8Array[]): Promise<BoardWithMetadataAndId[]> {
        try {
            const linkBases = tokenIds.map(tokenIdToLinkBase);
            const incomingBoards = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_boards_from_links',
                payload: linkBases,
            }) as IncomingBoardWithMetadataAndId[]

            return incomingBoards.map(parseIncomingBoardWithMetadataAndId)
        } catch (e) {
            console.log("Error with getBoardsFromTokenIds", e?.data?.data || e)
        }
    }

    async getBoardsFromAllMyMoves(): Promise<BoardWithMetadata[]> {
        try {
            const incomingBoards = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_boards_from_all_my_moves',
                payload: null,
            }) as IncomingBoardWithMetadata[]

            return incomingBoards.map(parseIncomingBoardWithMetadata)
        } catch (e) {
            console.log("Error with getBoardsFromAllMyMoves", e?.data?.data || e)
        }
    }

    async tokenIdToMetadata(tokenid: string): Promise<String> {
        try {
            return await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'token_id_to_metadata',
                payload: tokenid,
            }) as String
        } catch (e) {
            console.log(e?.data?.data)
            console.log(e)
        }
    }

    // participation proof
    async buildAgentParticipation(): Promise<ParticipationProof> {
        try {
            const record = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'build_agent_participation',
                payload: null,
            }) as Record
            return record as any as ParticipationProof
        } catch (e) {
            console.log(e?.data?.data || e)
        }
    }

    async createParticipationProof(participationProof: ParticipationProof): Promise<Record> {
        const _participationProof = transformParticipationProof(participationProof);
        try {
            return await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'create_participation_proof',
                payload: _participationProof,
            }) as Record
        } catch (e) {
            console.log(e?.data?.data || e)
        }
    }

    async getSignedParticipation(evmKey: Address): Promise<AgentParticipation> {
        try {
            const participation = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'get_signed_participation',
                payload: Array.from(hexToBytes(evmKey)),
            }) as AgentParticipation
            return participation
        } catch (e) {
            console.log(e?.data?.data || e)
        }
    }

    async boardToPng(board: Board, boardSize: "Small" | "Large" = "Large"): Promise<string> {
        try {
            const img = await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'board_to_png',
                payload: { board: { tiles: board }, board_size: boardSize },
            }) as string
            return img
        } catch (e) {
            console.log(e?.data?.data || e)
        }
    }

    async intializeMasks(): Promise<void> {
        try {
            await this.client.callZome({
                cap_secret: null,
                role_name,
                zome_name,
                fn_name: 'initialize_masks',
                payload: null,
            })
        } catch (e) {
            console.log(e?.data?.data || e)
        }
    }
}