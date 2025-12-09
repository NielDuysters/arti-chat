import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import { useChat } from "../../hooks/useChat";

import Message from "./Message";
import ChatInput from "./ChatInput";

import "./ChatWindow.scss";

export default function ChatWindow({ activeContact }) {
    const { messages, sendMessage } = useChat(activeContact);

    if (!activeContact) {
        return null;
    }

    return (
        <div className="screen screen--chat chat">
            <div className="chat__top">
                <div className="chat__contact-info">
                    <div className="chat__contact-info__image"></div>
                    <span className="chat__contact-info__nickname">{activeContact.nickname}</span>
                </div>
            </div>

            <div className="chat__messages">
                {messages.length === 0 && <p>No messages yet.</p>}

                {messages.map((msg) => (
                    <Message key={msg.timestamp} message={msg} />
                ))}
            </div>

            <ChatInput sendMessage={sendMessage} />
        </div>
    );
}

