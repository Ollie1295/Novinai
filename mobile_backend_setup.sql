-- Mobile Backend Database Schema
-- Run on VPS PostgreSQL to add mobile device support

-- Device registrations table
CREATE TABLE IF NOT EXISTS mobile_devices (
    device_id UUID PRIMARY KEY,
    home_id TEXT NOT NULL,
    platform TEXT CHECK (platform IN ('ios','android')) NOT NULL,
    push_token TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ,
    battery_level FLOAT,
    thermal_state TEXT DEFAULT 'nominal',
    UNIQUE(device_id, home_id)
);

-- Per-home mobile preferences
CREATE TABLE IF NOT EXISTS mobile_prefs (
    home_id TEXT PRIMARY KEY,
    process_on_battery BOOLEAN DEFAULT TRUE,
    max_px_small INTEGER DEFAULT 384,
    lite_timeout_ms INTEGER DEFAULT 300,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_mobile_devices_home_id ON mobile_devices(home_id);
CREATE INDEX IF NOT EXISTS idx_mobile_devices_last_seen ON mobile_devices(last_seen_at);
CREATE INDEX IF NOT EXISTS idx_mobile_devices_platform ON mobile_devices(platform);

-- Update trigger for mobile_prefs
CREATE OR REPLACE FUNCTION update_mobile_prefs_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_mobile_prefs_updated_at ON mobile_prefs;
CREATE TRIGGER trigger_mobile_prefs_updated_at
    BEFORE UPDATE ON mobile_prefs
    FOR EACH ROW
    EXECUTE FUNCTION update_mobile_prefs_timestamp();

-- Sample data for testing
INSERT INTO mobile_prefs (home_id) VALUES ('test_home_1') ON CONFLICT DO NOTHING;

COMMENT ON TABLE mobile_devices IS 'Registered mobile devices for push notifications and on-device processing';
COMMENT ON TABLE mobile_prefs IS 'Per-home preferences for mobile processing behavior';
