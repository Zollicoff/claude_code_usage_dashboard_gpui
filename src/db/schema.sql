-- Leads table for tracking all processed leads
CREATE TABLE IF NOT EXISTS leads (
    id TEXT PRIMARY KEY,
    property_id TEXT NOT NULL,
    property_address TEXT NOT NULL,
    owner_name TEXT NOT NULL,
    county TEXT NOT NULL,
    state TEXT NOT NULL,
    property_type TEXT NOT NULL,
    equity_percent REAL,
    ownership_years REAL NOT NULL,
    estimated_value REAL,
    qualification_score REAL NOT NULL,
    is_absentee BOOLEAN NOT NULL,
    is_distressed BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,
    pipedrive_id TEXT,
    sync_status TEXT NOT NULL,
    sync_error TEXT,
    last_synced_at TIMESTAMP,
    UNIQUE(property_address, owner_name)
);

-- Index for duplicate detection
CREATE INDEX IF NOT EXISTS idx_leads_property_owner
ON leads(property_address, owner_name);

-- Index for status queries
CREATE INDEX IF NOT EXISTS idx_leads_sync_status
ON leads(sync_status);

-- Index for date queries
CREATE INDEX IF NOT EXISTS idx_leads_created_at
ON leads(created_at);

-- Stats table for integration metrics
CREATE TABLE IF NOT EXISTS integration_stats (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    total_properties_fetched INTEGER DEFAULT 0,
    total_qualified_leads INTEGER DEFAULT 0,
    total_synced_to_pipedrive INTEGER DEFAULT 0,
    total_duplicates_skipped INTEGER DEFAULT 0,
    total_failures INTEGER DEFAULT 0,
    last_run TIMESTAMP,
    last_success TIMESTAMP,
    last_error TEXT,
    qualification_rate REAL DEFAULT 0.0,
    sync_success_rate REAL DEFAULT 0.0
);

-- Initialize stats row
INSERT OR IGNORE INTO integration_stats (id) VALUES (1);

-- Sync history table for detailed logging
CREATE TABLE IF NOT EXISTS sync_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    started_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    properties_fetched INTEGER DEFAULT 0,
    leads_qualified INTEGER DEFAULT 0,
    leads_synced INTEGER DEFAULT 0,
    duplicates_skipped INTEGER DEFAULT 0,
    errors_count INTEGER DEFAULT 0,
    status TEXT NOT NULL,
    error_message TEXT
);

-- Index for sync history queries
CREATE INDEX IF NOT EXISTS idx_sync_history_run_id
ON sync_history(run_id);

CREATE INDEX IF NOT EXISTS idx_sync_history_started_at
ON sync_history(started_at);
