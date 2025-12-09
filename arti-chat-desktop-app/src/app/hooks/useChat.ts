import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Message {
    body: string;
    timestamp: number;
    is_incoming: boolean;
}

export function useChat(activeContact) {
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
    }, []);

    // Send message.
    const sendMessage = async (text: string) => {
        if (!activeContact) {
            return;
        }

        // Push to chat to avoid slow UI.
        setMessages((prev) => [
            ...prev,
            {
                body: text,
                timestamp: Date.now(),
                is_incoming: false,
            }
        ])

        await invoke("send_message", {
            to: activeContact.onion_id,
            text: text,
        })

        await loadChat();
    }

    return {
        messages,
        sendMessage,
    };
}

