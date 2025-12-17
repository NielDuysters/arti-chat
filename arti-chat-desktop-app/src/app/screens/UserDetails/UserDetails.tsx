import { useState} from "react";

import Form from "../../components/forms/Form";
import updateUserForm from "../../formDefinitions/updateUserForm";
import { User, useUser } from "../../hooks/useUser";
import TextArea from "../../components/forms/fields/TextArea";

import "../../components/forms/FormField.scss";
import "./UserDetails.scss";

export default function UserDetail()  {
    const [success, setSuccess] = useState<boolean | null>(null);
    const { user, updateUser } = useUser();

    if (!user) {
        return;
    }

    // Generate Base64 token of user info.
    const generateShareToken = (user) => {
        const userInfo = {
            "onion_id": user.onion_id,
            "public_key": user.public_key,
        }

        return window.btoa(JSON.stringify(userInfo))
    }

    return (
        <div className="screen screen--user-details">
            <h2>Your info</h2>
            <Form
                fields={updateUserForm(user)}
                onSubmit={async (values) => {
                    return await updateUser({
                        public_key: values.public_key,
                        private_key: values.private_key,
                    }); 
                }}
                success={success}
                setSuccess={setSuccess}
            />

            <h3>Your base64 share token</h3>
            <p>Safely send this token to the person you want to have your contact info, preferably use PGP.</p>
            <TextArea
                value={generateShareToken(user)}
                disabled={true} 
                onClick={() => navigator.clipboard.writeText(generateShareToken(user))}
            />
        </div>
    );
}
