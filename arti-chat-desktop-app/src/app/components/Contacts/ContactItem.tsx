import React from "react";
import Jdenticon from "react-jdenticon";
import { Contact } from "../../hooks/useContacts";
import "./ContactItem.scss";


export default function ContactItem({ contact, onClick }: { contact: Contact }) {

    const amountUnreadMessages = (amount) => {
        if (amount === 0) {
            return;
        }

        return (
            <span className="contact__unread-amount">{amount}</span>
        );
    }

    return (
        <div className="contact" onClick={onClick}>
            <div className="contact__image">
                <Jdenticon size="35" value={contact.nickname} />
            </div>
            <span>{contact.nickname}</span>
            {amountUnreadMessages(contact.amount_unread_messages)}
        </div>
    );
}
