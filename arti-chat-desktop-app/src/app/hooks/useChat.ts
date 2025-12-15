import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Message {
    id: number;
    body: string;
    timestamp: number;
    is_incoming: boolean;
    sent_status: boolean;
    verified_status: boolean;
}

export function useChat({activeContact, loadContacts}) {
    const [messages, setMessages] = useState<Message[]>([]);

    // Load chat.
    const loadChat = useCallback(async () => {
        if (!activeContact) {
            return;
        }

        const msgs = await invoke<Message[]>("load_chat", {
            onionId: activeContact.onion_id,
        });

        setMessages(msgs);
    }, [activeContact]);

    useEffect(() => {
        loadChat();
    }, [loadChat]);

    // Send message.
    const sendMessage = async (text: string) => {
        if (!activeContact) {
            return;
        }

        // Push to chat to avoid slow UI.
        const latestPreviousMessage = messages.at(-1);
        setMessages((prev) => [
            ...prev,
            {
                id: latestPreviousMessage ? latestPreviousMessage.id + 1 : 1,
                body: text,
                timestamp: Date.now(),
                is_incoming: false,
                sent_status: false,
                verified_status: true,
                optimistic: true,
            }
        ])

        await invoke("send_message", {
            to: activeContact.onion_id,
            text: text,
        })

        await loadChat();
    }

    // Listen for new messages.
    useEffect(() => {
        const promise = listen("incoming-message", async (event) => {
            const data = JSON.parse(event.payload);
            await loadChat();
            await loadContacts();
        });

        return () => {
            promise.then((p) => p());
        };
    }, [activeContact, loadChat])

    return {
        messages,
        sendMessage,
    };
}

