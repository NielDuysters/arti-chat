import React from "react";
import { useState } from "react";

import Form from "../../components/forms/Form";
import { addContactForm } from "../../formDefinitions/addContactForm";
import { useContacts } from "../../hooks/useContacts";

import "./AddContact.scss";

export default function AddContact()  {
    const { addContact } = useContacts();
    const [success, setSuccess] = useState<boolean | null>(null);

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
