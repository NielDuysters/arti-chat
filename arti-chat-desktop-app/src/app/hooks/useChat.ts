import { useEffect, useLayoutEffect, useState, useCallback } from "react";
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

const BATCH_SIZE = 25;

export function useChat({activeContact, loadContacts, messageBatchNumber}) {
    const [messages, setMessages] = useState<Message[]>([]);

    // Load chat.
    const loadChat = useCallback(async () => {
        if (!activeContact) {
            return;
        }

        const msgs = await invoke<Message[]>("load_chat", {
            onionId: activeContact.onion_id,
            offset: 0,
            limit: messageBatchNumber * BATCH_SIZE
        });

        setMessages(msgs.reverse());
    }, [activeContact, messageBatchNumber]);

    useEffect(() => {
        loadChat();
    }, [loadChat]);

    useEffect(() => {
        loadChat();
    }, [messageBatchNumber]);
    
    // Send message.
    const sendMessage = async (text: string) => {
        if (!activeContact) {
            return;
        }

        // Push to chat to avoid slow UI.
        const message = {
            type: "Text",
            content: {
                text: text
            }
        };
        const latestPreviousMessage = messages.at(-1);
        setMessages((prev) => [
            ...prev,
            {
                id: latestPreviousMessage ? latestPreviousMessage.id + 1 : 1,
                body: JSON.stringify(message),
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
    
    // Send attachment.
    const sendAttachment = async (path: string) => {
        if (!activeContact) {
            return;
        }
        
        // Push to chat to avoid slow UI.
        const message = {
            type: "Text",
            content: {
                text: "Sending image " + path.split("/").pop() + "..."
            }
        };
        const latestPreviousMessage = messages.at(-1);
        setMessages((prev) => [
            ...prev,
            {
                id: latestPreviousMessage ? latestPreviousMessage.id + 1 : 1,
                body: JSON.stringify(message),
                timestamp: Date.now(),
                is_incoming: false,
                sent_status: false,
                verified_status: true,
                optimistic: true,
            }
        ])

        let response = await invoke("send_attachment", {
            to: activeContact.onion_id,
            path: path,
        })

        if (!response.success) {
            console.log("Error", response.error);
        }

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
    }, [loadContacts, loadChat])

    return {
        messages,
        sendMessage,
        sendAttachment,
    };
}

