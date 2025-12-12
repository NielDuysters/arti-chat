import { FieldType } from "../components/forms/FieldType";
import { FieldConfig } from "../components/forms/Form";

export const addContactForm: FieldConfig[] = [
    {
        name: "nickname",
        label: "Nickname",
        type: FieldType.Text,
        placeholder: "Alice",
        required: true,
    },
    {
        name: "onion_id",
        label: "Onion ID",
        type: FieldType.Text,
        placeholder: "abcdef.onion",
        required: true,
    },
    {
        name: "public_key",
        label: "Public key",
        type: FieldType.Text,
        placeholder: "fwejewfpewijfepwjfew",
        required: true,
    },
];
