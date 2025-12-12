import { FieldType } from "../components/forms/FieldType";
import { FieldConfig } from "../components/forms/Form";
import { User } from "../hooks/useUser";

export default function updateUserForm(user: User): FieldConfig[] {
    return [
        {
            name: "nickname",
            label: "Nickname",
            value: user.nickname,
            type: FieldType.Text,
            placeholder: "Alice",
            disabled: true,
        },
        {
            name: "onion_id",
            label: "Onion ID",
            value: user.onion_id,
            type: FieldType.Text,
            placeholder: "abcdef.onion",
            disabled: true,
        },
        {
            name: "public_key",
            label: "Public key",
            value: user.public_key,
            type: FieldType.Text,
            placeholder: "fwejewfpewijfepwjfew",
        },
        {
            name: "private_key",
            label: "Private key",
            value: user.private_key,
            type: FieldType.Text,
            placeholder: "fwejewfpewijfepwjfew",
        },
    ];
}
