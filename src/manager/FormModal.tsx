interface FormModalProps {
  title: string;
  onCancel: () => void;
  onSave: () => void;
  saveDisabled?: boolean;
  children: React.ReactNode;
}

function FormModal({ title, onCancel, onSave, saveDisabled, children }: FormModalProps) {
  return (
    <div className="form-overlay" onClick={onCancel}>
      <div className="form-dialog" onClick={(e) => e.stopPropagation()}>
        <h3>{title}</h3>
        {children}
        <div className="form-actions">
          <button className="btn btn-secondary" onClick={onCancel}>
            Cancel
          </button>
          <button className="btn btn-primary" onClick={onSave} disabled={saveDisabled}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}

export default FormModal;
