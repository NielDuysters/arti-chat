import "./Message.scss";

export default function Message({ message }) {
    const formatTimeFromTs = (ts) => {
        const date = new Date(ts * 1000);
        const hours = date.getHours().toString().padStart(2, "0");
        const minutes = date.getMinutes().toString().padStart(2, "0");
        return `${hours}:${minutes}`;
    }

    const statusIndicator = (sent_status) => {
        if (sent_status) {
            return (
                <img
                    className="message__sent-status"
                    alt="Sent successfully"
                    title="Sent successfully"
                    src="/src/assets/message-sent.png"
                />
            )
        }

        return (
            <img
                className="message__sent-status"
                alt="Message pending"
                title="Message pending"
                src="/src/assets/message-pending.png"
            />
        )
    }

    return (
        <div className={`message ${message.is_incoming ? "message--incoming" : "message--outgoing"} ${message.optimistic ? "message--optimistic" : ""}`}>
            <span className="message__body">{message.body}</span>
            <div className="message__info">
                <span className="message__timestamp">{!message.optimistic && formatTimeFromTs(message.timestamp)}</span>
                {!message.is_incoming && !message.optimistic && statusIndicator(message.sent_status)}
            </div>
        </div>
    );
}

