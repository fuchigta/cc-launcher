interface CrudTabLayoutProps {
  error: string | null;
  clearError: () => void;
  itemCount: number;
  itemLabel: string;
  newButtonLabel: string;
  emptyMessage: string;
  emptyButtonLabel: string;
  onNew: () => void;
  headers: string[];
  children: React.ReactNode;
}

function CrudTabLayout(props: CrudTabLayoutProps): React.ReactElement {
  const {
    error,
    clearError,
    itemCount,
    itemLabel,
    newButtonLabel,
    emptyMessage,
    emptyButtonLabel,
    onNew,
    headers,
    children,
  } = props;

  return (
    <div>
      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button type="button" onClick={clearError}>
            &times;
          </button>
        </div>
      )}
      <div className="toolbar">
        <span>
          {itemCount} {itemLabel}(s)
        </span>
        <div className="toolbar-actions">
          <button className="btn btn-primary" onClick={onNew}>
            {newButtonLabel}
          </button>
        </div>
      </div>

      {itemCount === 0 ? (
        <div className="empty-state">
          <p>{emptyMessage}</p>
          <button className="btn btn-primary" onClick={onNew}>
            {emptyButtonLabel}
          </button>
        </div>
      ) : (
        <table className="data-table">
          <thead>
            <tr>
              {headers.map((h) => (
                <th key={h}>{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>{children}</tbody>
        </table>
      )}
    </div>
  );
}

export default CrudTabLayout;
