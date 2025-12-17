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
        name: "share_token",
        label: "Base64 share token",
        type: FieldType.TextArea,
        placeholder: "",
        required: true,
    },
];
