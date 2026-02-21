import { open } from "@tauri-apps/plugin-dialog";

interface DirectoryInputProps {
  value: string;
  onChange: (value: string) => void;
}

function DirectoryInput({ value, onChange }: DirectoryInputProps) {
  return (
    <div className="form-group">
      <label>Working Directory</label>
      <div className="input-with-button">
        <input
          className="form-input"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="C:\path\to\project (optional)"
        />
        <button
          type="button"
          className="btn btn-secondary"
          onClick={async () => {
            const selected = await open({
              directory: true,
              multiple: false,
              defaultPath: value || undefined,
            });
            if (selected && typeof selected === "string") {
              onChange(selected);
            }
          }}
        >
          Browse
        </button>
      </div>
    </div>
  );
}

export default DirectoryInput;
