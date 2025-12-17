import { FieldConfig } from "./Form";
import { FieldType } from "./FieldType";
import TextInput from "./fields/TextInput";
import TextArea from "./fields/TextArea";
import "./FormField.scss";

export default function FormField({
    config,
    value,
    onChange,
}: {
    config: FieldConfig;
    value: any;
    onChange: (name: string, value: any) => void;
}) {
    const handleChange = (val) => onChange(config.name, val);

    switch (config.type) {
        case FieldType.Text:
            return <TextInput label={config.label} placeholder={config.placeholder} value={value} required={config.required} disabled={config.disabled} onChange={handleChange} />;
        case FieldType.TextArea:
            return <TextArea label={config.label} placeholder={config.placeholder} value={value} required={config.required} disabled={config.disabled} onChange={handleChange} />;
        default:
            return null;
    }
}

