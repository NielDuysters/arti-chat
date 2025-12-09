import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useChat } from "../../hooks/useChat";

export default function ChatWindow({ activeContact }) {
    const { messages } = useChat(activeContact);

    if (!activeContact) {
        return null;
    }

    return (
        <div className="screen screen--chat">
            <span>{activeContact.nickname}</span>

            <div className="screen--chat__messages">
                {messages.length === 0 && <p>No messages yet.</p>}

                {messages.map((msg) => (
                    <div key={msg.timestamp}>
                        {msg.body}
                    </div>
                ))}
            </div>
        </div>
    );
}

