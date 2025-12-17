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

    // Decode Base64 token to get user info.
    const decodeShareToken = (token) => {
        const decoded = window.atob(token);
        const json = JSON.parse(decoded);

        return {
            "onion_id": json.onion_id,
            "public_key": json.public_key,
        }
    }

    return (
        <div className="screen screen--add-contact">
            <h2>New contact</h2>
            <Form
                fields={addContactForm}
                onSubmit={async (values) => {
                    let contactInfo;
                    try {
                        contactInfo = decodeShareToken(values.share_token);
                    } catch (err) {
                        return false;
                    }

                    return await addContact({
                        nickname: values.nickname,
                        onion_id: contactInfo.onion_id,
                        public_key: contactInfo.public_key,
                    });
                }}
                success={success}
                setSuccess={setSuccess}
            />
        </div>
    );
}
