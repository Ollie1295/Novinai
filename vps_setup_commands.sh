#!/bin/bash
# VPS Configuration for PC Deep Worker Access

echo "🔧 Configuring VPS for PC Deep Worker Access"
echo "VPS Public IP: 95.179.193.224"

# 1. Configure Redis to accept external connections
echo "📡 Configuring Redis..."
sudo sed -i 's/bind 127.0.0.1/bind 0.0.0.0/' /etc/redis/redis.conf
sudo sed -i 's/protected-mode yes/protected-mode no/' /etc/redis/redis.conf
sudo systemctl restart redis

# 2. Configure PostgreSQL to accept external connections
echo "🗄️ Configuring PostgreSQL..."
sudo -u postgres psql -c "ALTER USER novin PASSWORD 'novin';"

# Edit postgresql.conf
sudo sed -i "s/#listen_addresses = 'localhost'/listen_addresses = '*'/" /etc/postgresql/*/main/postgresql.conf

# Edit pg_hba.conf to allow connections from your PC IP
# Replace YOUR_PC_IP with your actual PC's public IP
echo "host    all             novin           0.0.0.0/0               md5" | sudo tee -a /etc/postgresql/*/main/pg_hba.conf

sudo systemctl restart postgresql

# 3. Configure UFW firewall
echo "🔥 Configuring firewall..."
sudo ufw allow 6379/tcp comment "Redis for PC Deep Worker"
sudo ufw allow 5432/tcp comment "PostgreSQL for PC Deep Worker"
sudo ufw reload

# 4. Test connections
echo "🧪 Testing services..."
echo "Redis status:"
redis-cli ping

echo "PostgreSQL status:"
sudo -u postgres psql -c "SELECT version();"

echo "✅ VPS configuration complete!"
echo ""
echo "🔒 SECURITY NOTE:"
echo "Redis and PostgreSQL are now accessible from the internet."
echo "Consider setting up a VPN or restricting access to your PC's IP only."
echo ""
echo "To restrict to your PC IP only, run:"
echo "sudo ufw delete allow 6379/tcp"
echo "sudo ufw delete allow 5432/tcp"
echo "sudo ufw allow from YOUR_PC_PUBLIC_IP to any port 6379"
echo "sudo ufw allow from YOUR_PC_PUBLIC_IP to any port 5432"
