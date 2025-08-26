#!/bin/bash

# Fix IncidentStore to be Debug + Clone
sed -i 's/pub struct IncidentStore { pub incidents: HashMap<(String,String), Incident>, pub ttl_secs: f64, pub id_counter: u64 }/#[derive(Debug, Clone)]\npub struct IncidentStore { pub incidents: HashMap<(String,String), Incident>, pub ttl_secs: f64, pub id_counter: u64 }/' src/thinking/incident_engine.rs

# Add Clone and Debug to other needed structs
sed -i 's/pub struct Incident {/#[derive(Debug, Clone)]\npub struct Incident {/' src/thinking/incident_engine.rs
sed -i 's/pub struct Evidence {/#[derive(Debug, Clone)]\npub struct Evidence {/' src/thinking/incident_engine.rs
sed -i 's/pub struct Event {/#[derive(Debug, Clone)]\npub struct Event {/' src/thinking/incident_engine.rs

# Fix ThinkingAIProcessor
sed -i 's/pub struct ThinkingAIProcessor {/#[derive(Debug, Clone)]\npub struct ThinkingAIProcessor {/' src/thinking/mod.rs

# Fix missing manager.rs file
echo "Creating missing manager.rs file..."

