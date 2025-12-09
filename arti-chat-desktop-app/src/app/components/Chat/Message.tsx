import "./Message.scss";

export default function Message({ message }) {
    const formatTimeFromTs = (ts) => {
        const date = new Date(ts * 1000);
        const hours = date.getHours().toString().padStart(2, "0");
        const minutes = date.getMinutes().toString().padStart(2, "0");
        return `${hours}:${minutes}`;
    }

    return (
        <div className={`message ${message.is_incoming ? "message--incoming" : "message--outgoing"}`}>
            <span className="message__body">{message.body}</span>
            <div className="message__info">
                <span className="message__timestamp">{formatTimeFromTs(message.timestamp)}</span>
            </div>
        </div>
    );
}

