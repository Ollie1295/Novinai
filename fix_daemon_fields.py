#!/usr/bin/env python3

with open('src/bin/daemon.rs', 'r') as f:
    content = f.read()

# Fix ActiveResponse struct fields
old_active_response = """active_response: ActiveResponse {
                automated_actions: ResponseExecutor::default(),
                escalation_protocols: EscalationProtocols::default(),
                threat_mitigation: MitigationStrategies::default(),
                adaptive_defenses: AdaptiveDefenseSystem::default(),
                response_coordinator: ResponseCoordinator::default(),
                ethical_constraints: EthicalConstraintEngine::default(),
            },"""

new_active_response = """active_response: ActiveResponse {
                countermeasure_engine: CountermeasureEngine::default(),
                response_coordinator: ResponseCoordinator::default(),
                automated_actions: AutomatedActionSystem::default(),
                escalation_protocols: EscalationProtocols::default(),
                threat_neutralization: ThreatNeutralization::default(),
            },"""

content = content.replace(old_active_response, new_active_response)

with open('src/bin/daemon.rs', 'w') as f:
    f.write(content)

print("Fixed daemon field assignments!")
