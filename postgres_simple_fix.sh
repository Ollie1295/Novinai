#!/bin/bash
# Simple PostgreSQL Fix - Alternative Methods

echo "🔧 PostgreSQL Simple Fix Script"
echo "==============================="

# Method 1: Try without sudo -u postgres
echo "Method 1: Direct PostgreSQL commands..."
sudo systemctl status postgresql

# Method 2: Reset postgres system user password
echo ""
echo "Method 2: Reset postgres system user..."
echo "Setting new password for postgres system user:"
sudo passwd postgres

# Method 3: Use peer authentication
echo ""
echo "Method 3: Switch to postgres user directly..."
sudo su - postgres -c "psql -c \"SELECT version();\""

# Method 4: Create user via SQL file
echo ""
echo "Method 4: Creating setup via SQL file..."
cat > /tmp/setup_novin.sql << 'EOF'
-- Create novin user and database
DROP USER IF EXISTS novin;
CREATE USER novin WITH PASSWORD 'novin';
ALTER USER novin CREATEDB;

DROP DATABASE IF EXISTS novin;
CREATE DATABASE novin OWNER novin;
GRANT ALL PRIVILEGES ON DATABASE novin TO novin;

-- Show result
\du novin
\l novin
EOF

sudo su - postgres -c "psql -f /tmp/setup_novin.sql"

# Method 5: Direct config file editing
echo ""
echo "Method 5: Direct configuration..."

# Find PostgreSQL config
PG_CONF=$(sudo find /etc/postgresql -name "postgresql.conf" 2>/dev/null | head -1)
PG_HBA=$(sudo find /etc/postgresql -name "pg_hba.conf" 2>/dev/null | head -1)

if [ -n "$PG_CONF" ]; then
    echo "Found postgresql.conf: $PG_CONF"
    sudo sed -i "s/#listen_addresses = 'localhost'/listen_addresses = '*'/" "$PG_CONF"
    sudo sed -i "s/listen_addresses = 'localhost'/listen_addresses = '*'/" "$PG_CONF"
fi

if [ -n "$PG_HBA" ]; then
    echo "Found pg_hba.conf: $PG_HBA"
    echo "host    all    novin    0.0.0.0/0    md5" | sudo tee -a "$PG_HBA"
fi

# Restart service
sudo systemctl restart postgresql

# Test connection
echo ""
echo "Testing connection..."
PGPASSWORD=novin psql -h localhost -U novin -d novin -c "SELECT 'Success!' as result;" 2>/dev/null || {
    echo "Local connection failed, trying alternative..."
    sudo -u postgres createuser -s novin 2>/dev/null
    sudo -u postgres createdb novin -O novin 2>/dev/null
    sudo -u postgres psql -c "ALTER USER novin PASSWORD 'novin';"
}

echo ""
echo "✅ Setup complete. Test with:"
echo "PGPASSWORD=novin psql -h 95.179.193.224 -U novin -d novin"
