import React from "react";
import { Contact } from "../../hooks/useContacts";
import "./ContactItem.scss";

export default function ContactItem({ contact, onClick }: { contact: Contact }) {
  return (
    <div className="contact" onClick={onClick}>
      <div className="contact__image"></div>
      <span>{contact.nickname}</span>
    </div>
  );
}
