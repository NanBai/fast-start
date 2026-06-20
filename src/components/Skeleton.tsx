export function Skeleton() {
  return (
    <div className="skeleton-list" aria-hidden="true">
      {[0, 1, 2].map((g) => (
        <div key={g} className="skeleton-group">
          {[0, 1].map((r) => (
            <div key={r} className="skeleton-row">
              <div style={{ display: "grid", gap: 6, flex: 1, minWidth: 0 }}>
                <div className="shimmer shimmer-line short" />
                <div className="shimmer shimmer-line" />
              </div>
              <div className="shimmer shimmer-circle" />
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}

