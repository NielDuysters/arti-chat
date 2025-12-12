import { useEffect, useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import Jdenticon from "react-jdenticon";

import { useChat } from "../../hooks/useChat";

import Message from "./Message";
import ChatInput from "./ChatInput";

import "./ChatWindow.scss";

export default function ChatWindow({ activeContact, setView }) {
    const { messages, sendMessage } = useChat(activeContact);

    const chatRef = useRef<HTMLDivElement>(null);
    const [autoScrollToBottom, setAutoScrollToBottom] = useState(true);

    if (!activeContact) {
        return null;
    }

    // Detect if user is scrolling in chat.
    const handleScroll = () => {
        const el = chatRef.current;
        if (!el) {
            return;
        }

        const userIsAtBottom = el.scrollHeight - el.scrollTop <= el.clientHeight + 20;
        setAutoScrollToBottom(userIsAtBottom);
    };

    // Automatically scroll to bottom at incoming message if user is
    // not scrolling older chats.
    useEffect(() => {
        const el = chatRef.current;
        if (!el) {
            return;
        }

        if (autoScrollToBottom) {
            el.scrollTop = el.scrollHeight;
        }
    }, [messages, autoScrollToBottom]);

    return (
        <div className="screen screen--chat chat">
            <div className="chat__top">
                <div className="chat__contact-info">
                    <div className="chat__contact-info__image">
                        <Jdenticon size="35" value={activeContact.nickname} />
                    </div>
                    <span className="chat__contact-info__nickname">{activeContact.nickname}</span>
                </div>
                <img
                    className="chat__top__details"
                    alt="Details"
                    src="/src/assets/dots.png"
                    onClick={() => {
                        setView("contact-details");
                    }}
                />
            </div>

            <div
                className="chat__messages"
                ref={chatRef}
                onScroll={handleScroll}
            >
                {messages.length === 0 && <p>No messages yet.</p>}

                {messages.map((msg) => (
                    <Message key={msg.timestamp} message={msg} />
                ))}
            </div>

            <ChatInput sendMessage={sendMessage} />
        </div>
    );
}

