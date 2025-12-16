import { useState, useRef } from "react";
import "./ChatInput.scss";

export default function ChatInput({ sendMessage }) {
    const [text, setText] = useState("");
    const textInputRef = useRef(null);

    // Send message.
    const handleSend = async () => {
        if  (text.trim().length === 0) {
            return;
        }

        await sendMessage(text.trim());
        setText("");
    }

    // Send at Enter-key press.
    const handleKeydown = (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    }

    // Grow textarea height if multiple lines.
    const handleKeyup = (e) => {
        if (/\r|\n/.exec(text)) {
            textInputRef.current.style.height = "80px";
        } else {
            textInputRef.current.style.height = "40px";
        }
    }

    return (
        <div className="chat-input">
            <div className="chat-input__inner">
                <textarea
                    ref={textInputRef}
                    className="chat-input__text"
                    value={text}
                    onChange={(e) => setText(e.target.value)}
                    onKeyDown={handleKeydown}
                    onKeyUp={handleKeyup}
                    placeholder="Type your message..."
                ></textarea>
                <img
                    src="/src/assets/send.png"
                    alt="Send message"
                    className="chat-input__send-btn"
                    onClick={handleSend}
                />
            </div>
        </div>
    );
}

