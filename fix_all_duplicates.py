#!/usr/bin/env python3
import re

with open('src/core/mod.rs', 'r') as f:
    content = f.read()

print("Original file length:", len(content.split('\n')))

# Step 1: Remove all duplicate struct definitions by finding their line ranges
lines = content.split('\n')

# Find ranges to remove (duplicates)
duplicates_to_remove = []

# Find duplicate CausalGraph around line 1415
for i, line in enumerate(lines):
    if i > 1400 and 'pub struct CausalGraph {' in line:
        # Find the end of this struct
        brace_count = 0
        start_line = i
        for j in range(i, len(lines)):
            if '{' in lines[j]:
                brace_count += lines[j].count('{')
            if '}' in lines[j]:
                brace_count -= lines[j].count('}')
                if brace_count == 0:
                    # Remove from start-1 (derive line) to end
                    duplicates_to_remove.append((start_line-1, j))
                    print(f"Found duplicate CausalGraph to remove: lines {start_line-1} to {j}")
                    break
        break

# Find duplicate TemporalPatterns around line 1954
for i, line in enumerate(lines):
    if i > 1950 and 'pub struct TemporalPatterns;' in line:
        duplicates_to_remove.append((i-1, i))  # Remove derive line and struct line
        print(f"Found duplicate TemporalPatterns to remove: lines {i-1} to {i}")
        break

# Find duplicate CausalNode around line 2032
for i, line in enumerate(lines):
    if i > 2030 and 'pub struct CausalNode;' in line:
        duplicates_to_remove.append((i-1, i))  # Remove derive line and struct line
        print(f"Found duplicate CausalNode to remove: lines {i-1} to {i}")
        break

# Find duplicate CausalEdge around line 2035
for i, line in enumerate(lines):
    if i > 2033 and 'pub struct CausalEdge;' in line:
        duplicates_to_remove.append((i-1, i))  # Remove derive line and struct line
        print(f"Found duplicate CausalEdge to remove: lines {i-1} to {i}")
        break

# Find duplicate NetworkTopology around line 2225
for i, line in enumerate(lines):
    if i > 2220 and 'pub struct NetworkTopology;' in line:
        duplicates_to_remove.append((i-1, i))  # Remove derive line and struct line
        print(f"Found duplicate NetworkTopology to remove: lines {i-1} to {i}")
        break

# Sort duplicates by line number in reverse order so we remove from the end first
duplicates_to_remove.sort(key=lambda x: x[0], reverse=True)

# Remove the duplicate ranges
for start, end in duplicates_to_remove:
    print(f"Removing lines {start} to {end}: {lines[start:end+1]}")
    del lines[start:end+1]

# Step 2: Add missing type definitions at the end of existing placeholder types
missing_types = """
/// Countermeasure execution engine
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CountermeasureEngine {
    pub active_countermeasures: Vec<String>,
    pub execution_queue: Vec<String>,
    pub effectiveness_metrics: HashMap<String, f64>,
}

/// Automated action system for responses
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutomatedActionSystem {
    pub available_actions: Vec<String>,
    pub action_history: Vec<String>,
    pub safety_constraints: Vec<String>,
}

/// Escalation protocols for threat response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EscalationProtocols {
    pub escalation_levels: Vec<String>,
    pub notification_targets: Vec<String>,
    pub response_timeouts: HashMap<String, u32>,
}

/// Threat neutralization system
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreatNeutralization {
    pub neutralization_methods: Vec<String>,
    pub success_rate: f64,
    pub active_neutralizations: Vec<String>,
}

/// Causal confounding factors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Confounder {
    pub confounder_id: String,
    pub strength: f64,
    pub affected_variables: Vec<String>,
}

/// Causal mediation analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mediator {
    pub mediator_id: String,
    pub mediation_strength: f64,
    pub pathway: Vec<String>,
}

/// Causal collision detection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collider {
    pub collider_id: String,
    pub collision_strength: f64,
    pub parent_variables: Vec<String>,
}
"""

# Find a good place to insert the missing types (before EventCorrelationEngine)
insertion_point = -1
for i, line in enumerate(lines):
    if 'pub struct EventCorrelationEngine {' in line:
        insertion_point = i - 1
        break

if insertion_point > 0:
    missing_lines = missing_types.strip().split('\n')
    lines = lines[:insertion_point] + missing_lines + lines[insertion_point:]
    print(f"Added missing type definitions before line {insertion_point}")

# Step 3: Fix any structural issues in the remaining content
content = '\n'.join(lines)

# Fix any remaining issues with CausalGraph field access
# Replace the problematic CausalGraph constructor
old_constructor = """CausalGraph {
            nodes: vec![],
            edges: vec![],
            confounders: vec![],
            mediators: vec![],
            colliders: vec![],
            interventions: vec![],
        }"""

new_constructor = """CausalGraph {
            nodes: vec![],
            edges: vec![],
            interventions: vec![],
        }"""

content = content.replace(old_constructor, new_constructor)

# Write the fixed content back
with open('src/core/mod.rs', 'w') as f:
    f.write(content)

print("Fixed all duplicate definitions and missing types!")
print("New file length:", len(content.split('\n')))
