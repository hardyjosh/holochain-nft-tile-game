import { Button } from "flowbite-svelte";
import { configureChains } from "@wagmi/core";
import { polygonMumbai } from "@wagmi/core/chains";
import { createConfig, account } from "svelte-wagmi-stores";
// this example also uses Web3Modal - you'll need to install this yourself
import { Web3Modal } from "@web3modal/html";
import {
    EthereumClient,
    w3mConnectors,
    w3mProvider,
} from "@web3modal/ethereum";
import { writable } from "svelte/store";

export const web3modal = writable(null);

export const initWeb3Modal = async () => {

    // all this boilerplate is from the web3modal docs
    const chains = [polygonMumbai];
    const projectId = import.meta.env.VITE_PROJECT_ID;

    const { publicClient } = configureChains(chains, [
        w3mProvider({ projectId }),
    ]);

    // except here we're using createConfig form this package instead of wagmi
    const wagmiConfig = createConfig({
        autoConnect: true,
        connectors: w3mConnectors({ projectId, chains }),
        publicClient,
    });

    const ethereumClient = new EthereumClient(wagmiConfig, chains);


    // necessary if you're using SSR, because there's no window for the modal to attach to
    const web3modalInst = new Web3Modal({ projectId }, ethereumClient);
    web3modalInst.setDefaultChain(polygonMumbai);
    web3modal.set(web3modalInst);

}