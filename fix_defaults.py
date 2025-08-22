#!/usr/bin/env python3
import re

with open('src/core/mod.rs', 'r') as f:
    content = f.read()

# List of types that need Default added
types_needing_default = [
    'AudioProcessor', 'IoTNetwork', 'MultiModalFusion', 'EmpathyEngine', 
    'SocialAwareness', 'EmotionalResponseGenerator', 'OnlineLearningEngine',
    'MetaLearningFramework', 'ReasoningExplainer', 'CounterfactualEngine',
    'ResponseExecutor', 'AdaptiveDefenseSystem'
]

for type_name in types_needing_default:
    # Pattern to find struct definitions that don't already have Default
    pattern = rf'(#\[derive\([^)]+\)\]\s*\npub struct {type_name})'
    
    def add_default(match):
        derive_line = match.group(1)
        if 'Default' not in derive_line:
            # Add Default to the derive attributes
            derive_line = derive_line.replace(')]\n', ', Default)]\n')
        return derive_line
    
    content = re.sub(pattern, add_default, content)

# Also fix the unused variable warning in daemon
content = content.replace(
    'execute_response(&mut self, event: &SecurityEvent',
    'execute_response(&mut self, _event: &SecurityEvent'
)

with open('src/core/mod.rs', 'w') as f:
    f.write(content)

print("Added Default implementations to all required types!")
