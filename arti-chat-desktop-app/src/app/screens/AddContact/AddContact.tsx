import React from "react";
import { useState, useEffect } from "react";

import Form from "../../components/forms/Form";
import { addContactForm } from "../../formDefinitions/addContactForm";
import { useContacts } from "../../hooks/useContacts";

export default function AddContact({contacts, setContacts, setActiveContact, setView})  {
    const { addContact } = useContacts({contacts: contacts, setContacts: setContacts});
    const [success, setSuccess] = useState<boolean | null>(null);

    useEffect(() => {
        if (success === true) {
            // Small delay to show status icon in submit button.
            const timer = setTimeout(() => {
                const newestContact = contacts.at(0);
                if (newestContact) {
                    setActiveContact(newestContact);
                    setView("chat");
                }
            }, 1025);

            return () => clearTimeout(timer);

        }
    }, [success, contacts, setActiveContact, setView]);

    return (
        <div className="screen screen--add-contact">
            <h2>New contact</h2>
            <Form
                fields={addContactForm}
                onSubmit={async (values) => {
                    return await addContact({
                        nickname: values.nickname,
                        onion_id: values.onion_id,
                        public_key: values.public_key,
                    });
                }}
                success={success}
                setSuccess={setSuccess}
            />
        </div>
    );
}
