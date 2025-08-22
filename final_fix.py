#!/usr/bin/env python3
import re

with open('src/core/mod.rs', 'r') as f:
    content = f.read()

print("Starting final fix...")

# Step 1: Remove placeholder structs that have full definitions
placeholder_patterns = [
    r'#\[derive\([^\]]+\)\]\s*pub struct Confounder;',
    r'#\[derive\([^\]]+\)\]\s*pub struct Mediator;', 
    r'#\[derive\([^\]]+\)\]\s*pub struct Collider;'
]

for pattern in placeholder_patterns:
    matches = list(re.finditer(pattern, content))
    for match in reversed(matches):  # Remove in reverse order
        print(f"Removing placeholder: {match.group()}")
        content = content[:match.start()] + content[match.end():]

# Step 2: Fix any remaining CausalGraph constructor issues
old_patterns = [
    r'confounders: vec!\[\],\s*',
    r'mediators: vec!\[\],\s*',
    r'colliders: vec!\[\],\s*'
]

for pattern in old_patterns:
    content = re.sub(pattern, '', content)
    print(f"Removed pattern: {pattern}")

# Step 3: Remove any remaining duplicates that might exist
content = re.sub(r'pub struct Confounder \{[^}]+\}(?=.*?pub struct Confounder \{)', '', content, flags=re.DOTALL)
content = re.sub(r'pub struct Mediator \{[^}]+\}(?=.*?pub struct Mediator \{)', '', content, flags=re.DOTALL)
content = re.sub(r'pub struct Collider \{[^}]+\}(?=.*?pub struct Collider \{)', '', content, flags=re.DOTALL)

with open('src/core/mod.rs', 'w') as f:
    f.write(content)

print("Final fix applied!")
