import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useClient() {
    // Reset Tor circuit.
    const resetTorCircuit = useCallback(async () : boolean => {
         return await invoke("reset_tor_circuit");
    }, []);

    return {
        resetTorCircuit
    };
}

