export default function TextArea({ label, placeholder, value, required, disabled, onChange, onClick }) {
  return (
    <div className="form-field form-field--textarea">
        <label className="form-field__label">{label}{required ? "*" : ""}</label>
        <textarea
            className="form-field--textarea__input"
            readOnly={disabled}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onClick={onClick}
            ></textarea>
    </div>
  );
}

