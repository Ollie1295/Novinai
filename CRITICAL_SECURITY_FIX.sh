#!/bin/bash
# CRITICAL SECURITY FIX - Run on VPS immediately

echo "🚨 CRITICAL: Securing exposed services"

# Get PC IP
PC_IP=$(curl -s https://ipinfo.io/ip)
echo "PC IP: $PC_IP"

# Lock down PostgreSQL and Redis
sudo ufw delete allow 5432/tcp 2>/dev/null
sudo ufw delete allow 6379/tcp 2>/dev/null
sudo ufw allow from $PC_IP to any port 5432
sudo ufw allow from $PC_IP to any port 6379
sudo ufw reload

echo "✅ PostgreSQL and Redis secured to PC only"
echo "Test connection: python test_vps_connection.py"
