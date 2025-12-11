import React from "react";
import Jdenticon from "react-jdenticon";
import { Contact } from "../../hooks/useContacts";
import "./ContactItem.scss";


export default function ContactItem({ contact, onClick }: { contact: Contact }) {
  return (
    <div className="contact" onClick={onClick}>
      <div className="contact__image">
        <Jdenticon size="35" value={contact.nickname} />
      </div>
      <span>{contact.nickname}</span>
    </div>
  );
}
