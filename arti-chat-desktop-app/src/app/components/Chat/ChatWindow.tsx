import { useEffect, useState, useCallback, useRef, useLayoutEffect, Fragment } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import Jdenticon from "react-jdenticon";

import { useChat } from "../../hooks/useChat";

import Message from "./Message";
import ChatInput from "./ChatInput";

import "./ChatWindow.scss";

export default function ChatWindow({ activeContact, loadContacts, setView, messages, sendMessage, sendAttachment, setMessageBatchNumber }) {
    const chatRef = useRef<HTMLDivElement>(null);
    const prevScrollHeightRef = useRef<number | null>(null);
    const [autoScrollToBottom, setAutoScrollToBottom] = useState(true);
    const [dayLabel, setDayLabel] = useState("today");
    const [dayLabelVisible, setDayLabelVisible] = useState(false);
    const hideDayLabelTimeout = useRef<number | null>(null);

    if (!activeContact) {
        return null;
    }

    const handleScroll = () => {
        const el = chatRef.current;
        if (!el) {
            return;
        }

        // Show day label when scrolling.
        updateDayLabel();
        // Hide again when user stops scrolling.
        if (hideDayLabelTimeout.current) {
            clearTimeout(hideDayLabelTimeout.current);
        }
        hideDayLabelTimeout.current = window.setTimeout(() => {
            setDayLabelVisible(false);
        }, 500);

        // Detect if user is scrolling in chat.
        const userIsAtBottom = el.scrollHeight - el.scrollTop <= el.clientHeight + 20;
        if (!userIsAtBottom) {
            setDayLabelVisible(true);
        }
        setAutoScrollToBottom(userIsAtBottom);

        // Detect if user is at top of chat.
        const userIsAtTop = el.scrollTop <= 20;
        if (userIsAtTop && prevScrollHeightRef.current === null) {
            prevScrollHeightRef.current = el.scrollHeight;
            setMessageBatchNumber(Math.floor(messages.length / 25) + 1);
        }
    };

    // Clean up hideDayLabelTimeout on unmount.
    useEffect(() => {
        return () => {
            if (hideDayLabelTimeout.current) {
                clearTimeout(hideDayLabelTimeout.current);
            }
        };
    }, []);

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

    useLayoutEffect(() => {
        const el = chatRef.current;
        if (!el) {
            return;
        }

        if (prevScrollHeightRef.current !== null) {
            const heightDiff = el.scrollHeight - prevScrollHeightRef.current;
            el.scrollTop = heightDiff;
            prevScrollHeightRef.current = null;
        }
    }, [messages]);

    // Get date of message.
    const dateOfMessage = (timestamp) => {
        return new Date(timestamp * 1000).toISOString().slice(0, 10);
    }

    // Format date for dayLabel.
    const formatMessageDate = (date) => {
        const dateWithoutTime = (d) => {
            const copy = new Date(d);
            copy.setHours(0, 0, 0, 0);
            return copy;
        }

        const today = dateWithoutTime(new Date());
        const messageDate = dateWithoutTime(date);

        const diffMs = today - messageDate;
        const diffDays = Math.round(diffMs / (1000 * 60 * 60 * 24));

        switch (diffDays) {
            case 0:
                return "today";
            case 1:
                return "yesterday";
            default:
                return messageDate.toLocaleDateString();
        }
    }

    // Get date of messages user is scrolling.
    const updateDayLabel = useCallback(() => {
      const container = chatRef.current;
      if (!container) {
          return;
      }

      const markers = container.querySelectorAll(".chat__date-mark");
      if (!markers.length) {
          return;
      }

      const containerTop = container.getBoundingClientRect().top - 100;

      let closestMarker = null;
      let closestDistance = Infinity;

      markers.forEach((marker) => {
        const rect = marker.getBoundingClientRect();
        const distance = Math.abs(rect.top - containerTop);

        if (distance < closestDistance && rect.bottom >= containerTop) {
          closestDistance = distance;
          closestMarker = marker;
        }
      });

      if (closestMarker) {
        const date = closestMarker.dataset.dateMark;
        if (date) {
          setDayLabel(formatMessageDate(new Date(date)));
        }
      }
    }, []);


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
                    src="/assets/dots.png"
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

                <div className={`chat__day-label ${dayLabelVisible ? "chat__day-label--visible" : ""}`}>{dayLabel}</div>
                
                {messages.length === 0 && <p>No messages yet.</p>}

                {(() => {
                    let lastDate: string | null = null;

                    return messages.map((msg) => {
                        const messageDate = dateOfMessage(msg.timestamp);
                        const showDateMark = messageDate !== lastDate;

                        lastDate = messageDate;

                        return (
                            <Fragment key={msg.id}>
                                {showDateMark && (
                                    <div
                                        className="chat__date-mark"
                                        data-date-mark={messageDate}
                                    />
                                )}
                                <Message message={msg} />
                            </Fragment>
                        );
                    });
                })()}
            </div>

            <ChatInput sendMessage={sendMessage} sendAttachment={sendAttachment} />
        </div>
    );
}

