import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Contact {
    onion_id: string;
    nickname: string;
    public_key: string;
    amount_unread_messages: number;
    last_viewed_at: number;
}

export function useContacts({contacts, setContacts}) {

    // Load contact list.
    const loadContacts = useCallback(async () => {
        const contacts = await invoke<Contact[]>("load_contacts");
        setContacts(contacts);
    }, []);

    // Add a new contact.
    const addContact = useCallback(
        async (contact: Contact): boolean => {
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
    
    // Update an existing contact.
    const updateContact = useCallback(
        async ({onion_id, nickname, public_key}: {onion_id: string, nickname: string | null, public_key: string | null}): boolean => {
            let response = await invoke("update_contact", {
                onionId: onion_id,
                nickname: nickname,
                publicKey: public_key,
            });

            await loadContacts();

            return response;
        },
        [loadContacts]
    );

    // Delete messages of contact.
    const deleteContactMessages = useCallback(
        async (onion_id): boolean => {
            let response = await invoke("delete_contact_messages", {
                onionId: onion_id,
            });

            return response;
        },
        []
    );
    
    // Delete contact.
    const deleteContact = useCallback(
        async (onion_id): boolean => {
            let response = await invoke("delete_contact", {
                onionId: onion_id,
            });

            await loadContacts();

            return response;
        },
        [loadContacts]
    );
    
    // Delete all contacts.
    const deleteAllContacts = useCallback(
        async (): boolean => {
            let response = await invoke("delete_all_contacts");
            await loadContacts();

            return response;
        },
        [loadContacts]
    );

    return {
        loadContacts,
        contacts,
        addContact,
        updateContact,
        deleteContactMessages,
        deleteContact,
        deleteAllContacts,
    };
}

