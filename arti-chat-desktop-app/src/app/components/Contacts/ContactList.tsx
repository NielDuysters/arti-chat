import React from "react";
import ContactItem from "./ContactItem";
import "./ContactList.scss";

export default function ContactList({contacts, setActiveContact, setView}) {
  return (
    <div className="contacts">
      <div className="contacts__header">
        <span className="contacts__title">Chats</span>
        <img 
            className="contacts__new-btn"
            src="/src/assets/edit.png"
            onClick={() => {
                setActiveContact(null);
                setView('add-contact');
            }}
        />
      </div>

      <div className="contacts__list">
        
        {contacts.length === 0 && (<div className="contacts__empty">
                                   No contacts yet.

                                    <div
                                        className="contacts__empty__btn"
                                        onClick={() => {
                                            setView('add-contact');
                                        }}
                                        >
                                        <span>Add first contact</span>
                                    </div>
                                       </div>)}

        {contacts.map((c) => (
            <ContactItem
                key={c.onion_id}
                contact={c}
                onClick={() => {
                    c.amount_unread_messages = 0;
                    setActiveContact(c);
                    setView('chat');
                }}
            />
        ))}
      </div>
    </div>
  );
}

