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
                    src="/assets/message-sent.png"
                />
            )
        }

        return (
            <img
                className="message__sent-status"
                alt="Message pending"
                title="Message pending"
                src="/assets/message-pending.png"
            />
        )
    }

    const errorImage = (message) => {
        if (messageIsError(message)) {
            let message_body = JSON.parse(message.body);
            return (
                <div className="message__unverified-error">
                    <img
                        className="message__unverified-error__image"
                        alt="Error: "
                        title={message_body.content.message}
                        src="/assets/error.png"
                    />
                    <span>{message_body.content.message}</span>
                </div>
            )
        }
    }

    const messageContent = (message_content) => {
        let message = JSON.parse(message_content);
        if (message.type === "Text") {
            return message.content.text;
        }

        if (message.type === "Image") {
            const bytes = new Uint8Array(message.content.data);
            const blob = new Blob([bytes], { type: "image/jpeg" });
            const url = URL.createObjectURL(blob);

            return (
                <img className="message__attachment--image" src={url} alt="Error in image..." />
            )
        }

        if (message.type === "Error") {
            return message.content.message;
        }
    }

    const messageIsError = (message) => {
        if (message.is_incoming && !message.verified_status) {
            return true;
        }

        let message_body = JSON.parse(message.body);
        if (message_body.type === "Error") {
            return true;
        }

        return false;
    }

    return (
        <div
            className={`message ${message.is_incoming ? "message--incoming" : "message--outgoing"} ${message.optimistic ? "message--optimistic" : ""} ${messageIsError(message) ? "message--unverified" : ""}`}
            data-timestamp={message.timestamp}
        >
            {!message.optimistic && errorImage(message)}
            <span className="message__body">{messageContent(message.body)}</span>
            <div className="message__info">
                <span className="message__timestamp">{!message.optimistic && formatTimeFromTs(message.timestamp)}</span>
                {!message.is_incoming && !message.optimistic && statusIndicator(message.sent_status)}
            </div>
        </div>
    );
}

