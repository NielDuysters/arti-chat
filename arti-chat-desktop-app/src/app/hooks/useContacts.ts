import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Contact {
    onion_id: string;
    nickname: string;
    public_key: string;
}

export function useContacts() {
    const [contacts, setContacts] = useState<Contact[]>([]);

    // Load contacts once on mount.
    useEffect(() => {
        loadContacts();
    }, []);

    const loadContacts = useCallback(async () => {
        const list = await invoke<Contact[]>("load_contacts");
        setContacts(list);
    }, []);

    // Add a new contact.
    const addContact = useCallback(
        async (contact: Contact): boolean => {
            // Send to backend.
            let response = await invoke("add_contact", {
                nickname: contact.nickname,
                onionId: contact.onion_id,
                publicKey: contact.public_key,
            });

            await loadContacts();

            return response;
        },
        [loadContacts]
    );

    return {
        contacts,
        addContact,
    };
}

