import { useState} from "react";

import Form from "../../components/forms/Form";
import { Contact } from "../../hooks/useContacts";
import updateContactForm from "../../formDefinitions/updateContactForm";
import { useContacts } from "../../hooks/useContacts";

export default function ContactDetails({activeContact} : {activeContact: Contact} )  {
    const { updateContact } = useContacts();
    const [success, setSuccess] = useState<boolean | null>(null);

    return (
        <div className="screen screen--contact-details">
            <h2>{activeContact.nickname}</h2>
            <Form
                fields={updateContactForm(activeContact)}
                onSubmit={async (values) => {
                    return await updateContact({
                        onion_id: activeContact.onion_id,
                        nickname: values.nickname,
                        public_key: values.public_key,
                    }); 
                }}
                success={success}
                setSuccess={setSuccess}
            />
        </div>
    );
}
