export default function TextInput({ label, placeholder, value, required, onChange }) {
  return (
    <div className="form-field form-field--textinput">
      <label className="form-field__label">{label}{required ? "*" : ""}</label>
      <input
        className="form-field__input"
        type="text"
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}

