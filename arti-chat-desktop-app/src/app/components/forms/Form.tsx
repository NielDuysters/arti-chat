import React from "react";
import { useState, FormEvent } from "react";

import { FieldType } from "./FieldType";
import  FormField from "./FormField";

import "./Form.scss";

export interface FieldConfig {
    name: string;
    label: string;
    value?: string;
    type: FieldType;
    placeholder?: string;
    required?: boolean;
}

export interface FormProps {
    fields: FieldConfig[];
    onSubmit: (values: Record<string, any>) => void;
}

export default function Form({ fields, onSubmit }: FormProps) {
    const [values, setValues] = useState<Record<string, any>>({});

    const updateValue = (name: string, value: any) => {
        setValues((v) => ({ ...v, [name]: value }));
    };

    const handleSubmit = (e: FormEvent) => {
        e.preventDefault();
        onSubmit(values);
    };

    return (
        <form onSubmit={handleSubmit} className="form">
            {fields.map((f) => (
                <FormField
                key={f.name}
                config={f}
                value={values[f.name] ?? ""}
                onChange={updateValue}
                />
            ))}

            <button className="form__submit-btn" type="submit">Submit</button>
        </form>
    );
}
