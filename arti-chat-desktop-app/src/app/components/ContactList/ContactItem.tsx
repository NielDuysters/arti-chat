import React from "react";
import { Contact } from "../../hooks/useContacts";
import "./ContactItem.scss";

export default function ContactItem({ contact }: { contact: Contact }) {
  return (
    <div className="contact">
      <div className="contact-image"></div>
      <span>{contact.nickname}</span>
    </div>
  );
}
