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
    disabled?: boolean;
}

export interface FormProps {
    fields: FieldConfig[];
    onSubmit: (values: Record<string, any>) => void;
    success?: boolean;
    setSuccess: (value: boolean | null) => void;
}

export default function Form({ fields, onSubmit, success, setSuccess }: FormProps) {
    const [values, setValues] = useState<Record<string, any>>({});

    const updateValue = (name: string, value: any) => {
        setValues((v) => ({ ...v, [name]: value }));
    };

    const handleSubmit = async (e: FormEvent) => {
        e.preventDefault();
        const result = await onSubmit(values);
        setSuccess(result);
    };

    return (
        <form onSubmit={handleSubmit} className="form">
            {fields.map((f) => (
                <FormField
                key={f.name}
                config={f}
                value={values[f.name] ?? f.value}
                onChange={updateValue}
                />
            ))}

            <button
                className="form__submit-btn"
                type="submit"
            >
                {success === true &&
                    <img
                        className="form__submit-btn__status"
                        alt="Success"
                        src="/src/assets/button-success.png"
                    />
                }
                {success === false &&
                    <img
                        className="form__submit-btn__status"
                        alt="Failed"
                        src="/src/assets/button-failed.png"
                    />
                }
                Submit
            </button>
        </form>
    );
}
