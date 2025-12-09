import { useState } from "react";
import "./ChatInput.scss";

export default function ChatInput({ sendMessage }) {
    const [text, setText] = useState("");

    const handleSend = async () => {
        if  (text.trim().length === 0) {
            return;
        }

        await sendMessage(text.trim());
        setText("");
    }

    return (
        <div className="chat-input">
            <textarea
                className="chat-input__text"
                value={text}
                onChange={(e) => setText(e.target.value)}
                placeholder="Type your message..."
            ></textarea>
            <img
                src="/src/assets/send.png"
                alt="Send message"
                className="chat-input__send-btn"
                onClick={handleSend}
            />
        </div>
    );
}

