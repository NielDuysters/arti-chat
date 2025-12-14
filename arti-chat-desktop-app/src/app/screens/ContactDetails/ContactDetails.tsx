import { useState} from "react";

import Form from "../../components/forms/Form";
import { Contact } from "../../hooks/useContacts";
import updateContactForm from "../../formDefinitions/updateContactForm";
import { useContacts } from "../../hooks/useContacts";
import Action from "../../components/Action/Action";
import { ActionType } from "../../components/Action/ActionType";

export default function ContactDetails({activeContact} : {activeContact: Contact} )  {
    const { updateContact, deleteContactMessages } = useContacts();
    const [addContactSuccess, setAddContactSuccess] = useState<boolean | null>(null);
    const [deleteContactMessagesSuccess, setDeleteContactMessagesSuccess] = useState<boolean | null>(null);

    return (
        <div className="screen screen--contact-details">
            <h2>{activeContact.nickname}</h2>
            <h3>Info</h3>
            <Form
                fields={updateContactForm(activeContact)}
                onSubmit={async (values) => {
                    return await updateContact({
                        onion_id: activeContact.onion_id,
                        nickname: values.nickname,
                        public_key: values.public_key,
                    }); 
                }}
                success={addContactSuccess}
                setSuccess={setAddContactSuccess}
            />

            <h3>Actions</h3>
            <Action
                label="Delete messages"
                description="Delete all messages with this contact."
                actionType={ActionType.Delete}
                onClick={async () => {
                    const success = await deleteContactMessages(activeContact.onion_id);
                    setDeleteContactMessagesSuccess(success);
                }}
                success={deleteContactMessagesSuccess}
            />

        </div>
    );
}
