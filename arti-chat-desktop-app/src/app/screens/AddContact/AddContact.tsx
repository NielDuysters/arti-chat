import React from "react";

import Form from "../../components/forms/Form";
import { addContactForm } from "../../formDefinitions/addContactForm";

import "./AddContact.scss";

export default function AddContact()  {
    return (
        <div className="screen screen--add-contact">
            <h2>New contact</h2>
            <Form
                fields={addContactForm} 
                onSubmit={(values) => {
                    console.log("submitted: ", values)
                }}
            />
        </div>
    );
}
