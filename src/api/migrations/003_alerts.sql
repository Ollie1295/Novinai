-- Alerts table
CREATE TABLE IF NOT EXISTS alerts (
    id TEXT PRIMARY KEY,
    home_id TEXT NOT NULL,
    camera_id TEXT,
    alert_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    probability REAL NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at DATETIME,
    FOREIGN KEY (home_id) REFERENCES homes(id)
);
