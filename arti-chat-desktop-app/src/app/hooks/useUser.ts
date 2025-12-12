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

    const loadUser = useCallback(async () => {
        const user = await invoke<User>("load_user");
        console.log(user)
        setUser(user);
    }, []);


    return {
        user,
    };
}

