import { FieldType } from "../components/forms/FieldType";
import { FieldConfig } from "../components/forms/Form";
import { Contact } from "../hooks/useContacts";

export default function updateContactForm(contact: Contact): FieldConfig[] {
    return [
        {
            name: "nickname",
            label: "Nickname",
            value: contact.nickname,
            type: FieldType.Text,
            placeholder: "Alice",
            required: true,
        },
        {
            name: "onion_id",
            label: "Onion ID",
            value: contact.onion_id,
            type: FieldType.Text,
            placeholder: "abcdef.onion",
            disabled: true,
        },
        {
            name: "public_key",
            label: "Public key",
            value: contact.public_key,
            type: FieldType.Text,
            placeholder: "fwejewfpewijfepwjfew",
            required: true,
        },
    ];
}
