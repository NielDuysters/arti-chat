import { useState} from "react";

import Form from "../../components/forms/Form";
import updateUserForm from "../../formDefinitions/updateUserForm";
import { User, useUser } from "../../hooks/useUser";

export default function UserDetail()  {
    const [success, setSuccess] = useState<boolean | null>(null);
    const { user, updateUser } = useUser();

    if (!user) {
        return;
    }

    return (
        <div className="screen screen--contact-details">
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
        </div>
    );
}
