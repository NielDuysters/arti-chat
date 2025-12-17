import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useClient() {
    // Reset Tor circuit.
    const resetTorCircuit = useCallback(async () : boolean => {
         return await invoke("reset_tor_circuit");
    }, []);

    // Get config value.
    const getConfigValue = useCallback(async (key) : string => {
         return await invoke("get_config_value", {
             key: key,
         });
    }, []);
    
    // Set config value.
    const setConfigValue = useCallback(async (key, value) => {
         await invoke("set_config_value", {
             key: key,
             value: value,
         });
    }, []);

    return {
        resetTorCircuit,
        getConfigValue,
        setConfigValue,
    };
}

