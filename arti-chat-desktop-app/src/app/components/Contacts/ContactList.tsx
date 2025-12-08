import React from "react";
import { useContacts } from "../../hooks/useContacts";
import ContactItem from "./ContactItem";
import "./ContactList.scss";

export default function ContactList() {
  const { contacts } = useContacts();

  return (
    <div className="contacts">
      <div className="contacts__header">
        <span className="contacts__title">Chats</span>
        <img className="contacts__new-btn" src="/src/assets/edit.png" />
      </div>

      <div className="contacts__list">
        {contacts.map((c) => (
          <ContactItem key={c.onion} contact={c} />
        ))}
      </div>
    </div>
  );
}

