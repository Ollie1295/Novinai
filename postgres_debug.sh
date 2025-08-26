#!/bin/bash
# PostgreSQL Debug and Fix Script for VPS

echo "🔍 PostgreSQL Debug and Configuration Script"
echo "============================================="

# Check if PostgreSQL is installed and running
echo "1. Checking PostgreSQL service status..."
sudo systemctl status postgresql --no-pager || {
    echo "❌ PostgreSQL service not running or not installed"
    echo "Installing PostgreSQL..."
    sudo apt update
    sudo apt install -y postgresql postgresql-contrib
    sudo systemctl start postgresql
    sudo systemctl enable postgresql
}

# Check PostgreSQL version and location
echo ""
echo "2. PostgreSQL version and paths..."
sudo -u postgres psql -c "SELECT version();" 2>/dev/null || {
    echo "❌ Cannot connect to PostgreSQL as postgres user"
    echo "Trying to reset postgres user..."
    sudo passwd postgres
}

PG_VERSION=$(sudo -u postgres psql -t -c "SHOW server_version_num;" 2>/dev/null | tr -d ' ')
if [ ! -z "$PG_VERSION" ]; then
    PG_MAJOR=$(echo $PG_VERSION | cut -c1-2)
    echo "PostgreSQL version: $PG_MAJOR"
    PG_CONFIG_DIR="/etc/postgresql/$PG_MAJOR/main"
else
    echo "Finding PostgreSQL config directory..."
    PG_CONFIG_DIR=$(sudo find /etc/postgresql -name "postgresql.conf" -type f | head -1 | dirname)
fi

echo "Config directory: $PG_CONFIG_DIR"

# Create/configure novin user and database
echo ""
echo "3. Setting up novin user and database..."
sudo -u postgres psql << EOF
-- Drop and recreate user if exists
DROP USER IF EXISTS novin;
CREATE USER novin WITH PASSWORD 'novin';
ALTER USER novin CREATEDB;

-- Drop and recreate database if exists  
DROP DATABASE IF EXISTS novin;
CREATE DATABASE novin OWNER novin;

-- Grant all privileges
GRANT ALL PRIVILEGES ON DATABASE novin TO novin;

-- Show user info
\du novin
EOF

# Configure PostgreSQL for external connections
echo ""
echo "4. Configuring PostgreSQL for external access..."

# Backup original configs
sudo cp "$PG_CONFIG_DIR/postgresql.conf" "$PG_CONFIG_DIR/postgresql.conf.backup" 2>/dev/null
sudo cp "$PG_CONFIG_DIR/pg_hba.conf" "$PG_CONFIG_DIR/pg_hba.conf.backup" 2>/dev/null

# Update postgresql.conf
echo "Updating postgresql.conf..."
sudo sed -i "s/#listen_addresses = 'localhost'/listen_addresses = '*'/" "$PG_CONFIG_DIR/postgresql.conf"
sudo sed -i "s/listen_addresses = 'localhost'/listen_addresses = '*'/" "$PG_CONFIG_DIR/postgresql.conf"

# Update pg_hba.conf for external access
echo "Updating pg_hba.conf..."
sudo bash -c "cat >> $PG_CONFIG_DIR/pg_hba.conf" << EOF

# External access for novin user (added by setup script)
host    novin           novin           0.0.0.0/0               md5
host    all             novin           0.0.0.0/0               md5
EOF

# Restart PostgreSQL
echo ""
echo "5. Restarting PostgreSQL..."
sudo systemctl restart postgresql

# Test local connection
echo ""
echo "6. Testing local connection..."
PGPASSWORD=novin psql -h localhost -U novin -d novin -c "SELECT 'Connection successful!' as status;"

# Show current configuration
echo ""
echo "7. Current configuration:"
echo "Listen addresses:"
sudo grep "listen_addresses" "$PG_CONFIG_DIR/postgresql.conf" | grep -v "^#"

echo ""
echo "HBA rules for novin:"
sudo grep "novin" "$PG_CONFIG_DIR/pg_hba.conf" | grep -v "^#"

echo ""
echo "8. Firewall status:"
sudo ufw status | grep 5432 || echo "Port 5432 not in firewall rules"

echo ""
echo "✅ PostgreSQL configuration complete!"
echo ""
echo "Test external connection with:"
echo "PGPASSWORD=novin psql -h 95.179.193.224 -U novin -d novin -c 'SELECT NOW();'"
