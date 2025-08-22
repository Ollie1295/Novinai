#!/usr/bin/env python3
"""
First Awareness, Then Suppression - Delivery Scenario Demo
Shows how the AI security system provides initial awareness but suppresses redundant notifications
"""

from datetime import datetime, timedelta
from enum import Enum
from typing import Dict, List, Optional, NamedTuple
import json

class EventType(Enum):
    VEHICLE_APPROACH = "VehicleApproach"
    PERSON_DETECTED = "PersonDetected"
    DOOR_APPROACH = "DoorApproach"
    PACKAGE_DELIVERY = "PackageDelivery"

class AlertLevel(Enum):
    IGNORE = "Ignore"
    STANDARD = "Standard"
    ELEVATED = "Elevated"
    CRITICAL = "Critical"

class EventClassification(Enum):
    DELIVERY_SEQUENCE = "DeliverySequence"
    KNOWN_PERSON_MOVEMENT = "KnownPersonMovement"
    SUSPICIOUS_ACTIVITY = "SuspiciousActivity"

class NotificationDecision:
    pass

class Notify(NotificationDecision):
    def __init__(self, message: str, priority: str):
        self.message = message
        self.priority = priority
    
    def __str__(self):
        return f"🔔 NOTIFICATION ({self.priority}): {self.message}"

class Suppress(NotificationDecision):
    def __init__(self, reason: str, correlation_id: Optional[str] = None):
        self.reason = reason
        self.correlation_id = correlation_id
    
    def __str__(self):
        return f"🔇 SUPPRESSED: {self.reason} (Correlation: {self.correlation_id})"

class Summary(NotificationDecision):
    def __init__(self, message: str, event_count: int, correlation_id: str):
        self.message = message
        self.event_count = event_count
        self.correlation_id = correlation_id
    
    def __str__(self):
        return f"📋 SUMMARY: {self.message} ({self.event_count} events, ID: {self.correlation_id})"

class SecurityEvent(NamedTuple):
    id: str
    timestamp: datetime
    event_type: EventType
    location: str
    confidence: float
    alert_level: AlertLevel

class CorrelatedEvent:
    def __init__(self, primary_event_id: str, event_type: EventType, start_time: datetime, 
                 confidence: float, classification: EventClassification):
        self.primary_event_id = primary_event_id
        self.event_chain = [primary_event_id]
        self.event_type_sequence = [event_type]
        self.start_time = start_time
        self.last_update = start_time
        self.confidence_evolution = [confidence]
        self.classification = classification
        self.suppression_count = 0

class EventCorrelationEngine:
    def __init__(self):
        self.active_events: Dict[str, CorrelatedEvent] = {}
        self.correlation_window = timedelta(minutes=10)

    def correlate_event(self, event: SecurityEvent) -> Optional[str]:
        # Look for existing correlated events
        parent_id = self._find_correlatable_event(event)
        if parent_id:
            self._add_to_existing_correlation(parent_id, event)
            return parent_id
        elif self._is_sequence_initiator(event):
            self._start_new_correlation(event)
            return event.id
        return None

    def _find_correlatable_event(self, event: SecurityEvent) -> Optional[str]:
        for event_id, corr_event in self.active_events.items():
            if self._fits_sequence_pattern(event, corr_event):
                return event_id
        return None

    def _fits_sequence_pattern(self, event: SecurityEvent, corr_event: CorrelatedEvent) -> bool:
        if corr_event.classification == EventClassification.DELIVERY_SEQUENCE:
            last_type = corr_event.event_type_sequence[-1] if corr_event.event_type_sequence else None
            
            if event.event_type == EventType.PERSON_DETECTED and last_type == EventType.VEHICLE_APPROACH:
                return True
            elif event.event_type == EventType.DOOR_APPROACH and last_type == EventType.PERSON_DETECTED:
                return True
            elif event.event_type == EventType.PACKAGE_DELIVERY and last_type == EventType.DOOR_APPROACH:
                return True
        return False

    def _is_sequence_initiator(self, event: SecurityEvent) -> bool:
        return (event.event_type == EventType.VEHICLE_APPROACH or 
                (event.event_type == EventType.PERSON_DETECTED and event.confidence > 0.7))

    def _start_new_correlation(self, event: SecurityEvent):
        classification = self._classify_initial_event(event)
        corr_event = CorrelatedEvent(
            event.id, event.event_type, event.timestamp, event.confidence, classification
        )
        self.active_events[event.id] = corr_event

    def _add_to_existing_correlation(self, parent_id: str, event: SecurityEvent):
        if parent_id in self.active_events:
            corr_event = self.active_events[parent_id]
            corr_event.event_chain.append(event.id)
            corr_event.event_type_sequence.append(event.event_type)
            corr_event.last_update = event.timestamp
            corr_event.confidence_evolution.append(event.confidence)
            corr_event.suppression_count += 1

    def _classify_initial_event(self, event: SecurityEvent) -> EventClassification:
        if event.event_type == EventType.VEHICLE_APPROACH:
            return EventClassification.DELIVERY_SEQUENCE
        elif event.event_type == EventType.PERSON_DETECTED and event.confidence > 0.8:
            return EventClassification.KNOWN_PERSON_MOVEMENT
        else:
            return EventClassification.SUSPICIOUS_ACTIVITY

class NotificationStrategy:
    def __init__(self):
        self.awareness_threshold = 0.6
        self.suppression_enabled = True
        self.max_suppression_count = 5
        self.summary_enabled = True

    def decide_notification(self, event: SecurityEvent, 
                          correlation_engine: EventCorrelationEngine) -> NotificationDecision:
        # Check if this event is part of a correlated sequence
        if event.id in correlation_engine.active_events:
            corr_event = correlation_engine.active_events[event.id]
            return self._handle_correlated_event(event, corr_event)
        
        # Check if this event correlates to an existing sequence
        for parent_id, corr_event in correlation_engine.active_events.items():
            if event.id in corr_event.event_chain:
                return self._handle_sequence_event(event, corr_event, parent_id)
        
        # Standalone event
        return Notify(
            f"{event.alert_level.value} Alert: {event.event_type.value} at {event.location} "
            f"(Confidence: {event.confidence:.0%})",
            "Medium"
        )

    def _handle_correlated_event(self, event: SecurityEvent, 
                               corr_event: CorrelatedEvent) -> NotificationDecision:
        # First event in sequence - provide awareness
        if event.confidence >= self.awareness_threshold:
            return Notify(self._format_awareness_message(event, corr_event.classification), "Low")
        else:
            return Suppress("Below awareness threshold", event.id)

    def _handle_sequence_event(self, event: SecurityEvent, corr_event: CorrelatedEvent, 
                             parent_id: str) -> NotificationDecision:
        # Check if we should suppress based on sequence classification
        should_suppress = (
            corr_event.classification == EventClassification.DELIVERY_SEQUENCE and
            self.suppression_enabled and 
            corr_event.suppression_count < self.max_suppression_count
        )
        
        if should_suppress:
            # Check if this is the final event (for summary)
            if self._is_sequence_completion_event(event, corr_event) and self.summary_enabled:
                return Summary(
                    self._format_summary_message(corr_event),
                    len(corr_event.event_chain),
                    parent_id
                )
            else:
                return Suppress(f"Part of {corr_event.classification.value} sequence", parent_id)
        else:
            return Notify(
                f"{event.event_type.value} at {event.location} (Confidence: {event.confidence:.0%})",
                "Medium"
            )

    def _is_sequence_completion_event(self, event: SecurityEvent, 
                                    corr_event: CorrelatedEvent) -> bool:
        return (
            corr_event.classification == EventClassification.DELIVERY_SEQUENCE and
            (event.event_type == EventType.PACKAGE_DELIVERY or 
             ("street" in event.location and len(corr_event.event_chain) >= 3))
        )

    def _format_awareness_message(self, event: SecurityEvent, 
                                classification: EventClassification) -> str:
        if classification == EventClassification.DELIVERY_SEQUENCE:
            return "📦 Likely delivery activity detected. Monitoring..."
        elif classification == EventClassification.KNOWN_PERSON_MOVEMENT:
            return "👤 Known person detected on property. Tracking movement..."
        else:
            return "🔍 Activity detected. Analyzing..."

    def _format_summary_message(self, corr_event: CorrelatedEvent) -> str:
        duration = (datetime.now() - corr_event.start_time).total_seconds() / 60
        
        if corr_event.classification == EventClassification.DELIVERY_SEQUENCE:
            return f"✅ Delivery completed. Package delivered at front door. Duration: {duration:.0f}min"
        else:
            return f"📋 Activity sequence completed. {len(corr_event.event_chain)} events over {duration:.0f}min"

def main():
    print("🚀 First Awareness, Then Suppression - Delivery Scenario Demo\n")

    correlation_engine = EventCorrelationEngine()
    notification_strategy = NotificationStrategy()

    # Simulate delivery sequence events
    base_time = datetime.now()
    
    events = [
        # Event 1: Amazon van approaches (T+0s)
        SecurityEvent(
            id="event_001",
            timestamp=base_time,
            event_type=EventType.VEHICLE_APPROACH,
            location="street",
            confidence=0.8,
            alert_level=AlertLevel.IGNORE
        ),
        
        # Event 2: Driver exits with package (T+45s)
        SecurityEvent(
            id="event_002",
            timestamp=base_time + timedelta(seconds=45),
            event_type=EventType.PERSON_DETECTED,
            location="driveway",
            confidence=0.75,
            alert_level=AlertLevel.IGNORE
        ),
        
        # Event 3: Approaches front door (T+65s)
        SecurityEvent(
            id="event_003",
            timestamp=base_time + timedelta(seconds=65),
            event_type=EventType.DOOR_APPROACH,
            location="front_door",
            confidence=0.9,
            alert_level=AlertLevel.IGNORE
        ),
        
        # Event 4: Package delivery completed (T+95s)
        SecurityEvent(
            id="event_004",
            timestamp=base_time + timedelta(seconds=95),
            event_type=EventType.PACKAGE_DELIVERY,
            location="front_door",
            confidence=0.95,
            alert_level=AlertLevel.IGNORE
        ),
    ]

    print("🎬 Processing delivery sequence events:\n")

    for i, event in enumerate(events):
        elapsed = (event.timestamp - base_time).total_seconds()
        print(f"⏰ T+{elapsed:.0f}s - Event {i + 1}: {event.event_type.value} at {event.location}")

        # Correlate the event
        correlation_engine.correlate_event(event)

        # Decide on notification
        decision = notification_strategy.decide_notification(event, correlation_engine)

        # Display the decision
        print(f"   {decision}")
        print()

    print("📊 Final Correlation State:")
    for event_id, corr_event in correlation_engine.active_events.items():
        print(f"   Event Chain {event_id}: {len(corr_event.event_chain)} events, "
              f"Classification: {corr_event.classification.value}")
        print(f"   Confidence Evolution: {[f'{c:.2f}' for c in corr_event.confidence_evolution]}")
        print(f"   Suppression Count: {corr_event.suppression_count}")

    print("\n✨ Key Features Demonstrated:")
    print("   • Initial awareness notification for first event")
    print("   • Intelligent suppression of redundant notifications")
    print("   • Event correlation and sequence classification")
    print("   • Summary notification upon completion")
    print("   • User preference-driven behavior")
    
    print("\n🔍 How It Works:")
    print("   1. Van approach triggers initial awareness: 'Likely delivery activity detected'")
    print("   2. Subsequent events (person, door, package) are correlated and suppressed")
    print("   3. Final event triggers summary: 'Delivery completed'")
    print("   4. User gets awareness + completion summary, not 4 separate alerts")

if __name__ == "__main__":
    main()
