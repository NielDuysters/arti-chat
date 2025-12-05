import React from "react";
import { useContacts } from "../../hooks/useContacts";
import ContactItem from "./ContactItem";
import "./ContactList.scss";

export default function ContactList() {
  const { contacts } = useContacts();

  return (
    <div id="contact-list-wrapper">
      <div id="chats-top">
        <span id="chats-title">Chats</span>
        <img id="new-chat-button" src="/src/assets/edit.png" />
      </div>

      <div id="contact-list">
        {contacts.map((c) => (
          <ContactItem key={c.onion} contact={c} />
        ))}
      </div>
    </div>
  );
}

