import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface User {
    onion_id: string;
    nickname: string;
    public_key: string;
    private_key: string;
}

export function useUser() {
    const [user, setUser] = useState<User | null>(null);
    
    useEffect(() => {
        loadUser();
    }, []);

    // Load user.
    const loadUser = useCallback(async () => {
        const user = await invoke<User>("load_user");
        setUser(user);
    }, []);

    // Update user.
    const updateUser = useCallback(
        async ({public_key, private_key}: {public_key: string | null, private_key: string | null}): boolean => {
            return await invoke("update_user", {
                publicKey: public_key,
                privateKey: private_key,
            });
        },
        []
    );

    return {
        user,
        updateUser,
    };
}

