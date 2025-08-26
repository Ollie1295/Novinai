//! Fixed Evidence Test - Correct Threat Assessment

use insane_ai_security::thinking::{ThinkingAIProcessor, ThinkingAIConfig, Event, Evidence};
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("🔥 FIXED EVIDENCE TEST - CORRECT THREAT SEMANTICS");
    println!("=================================================");
    
    let config = ThinkingAIConfig::default();
    let mut thinking_ai = ThinkingAIProcessor::new(config);
    
    // CORRECTED EVIDENCE SEMANTICS:
    // POSITIVE LLR = EVIDENCE FOR THREAT/SUSPICIOUS
    // NEGATIVE LLR = EVIDENCE AGAINST THREAT/NORMAL
    
    let scenarios = vec![
        (
            "🚨 CRITICAL: Multiple Armed Intruders Breaking In",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "backyard_camera".to_string(),
                person_track: "multiple_unknown".to_string(),
                rang_doorbell: false,
                knocked: false,
                dwell_s: 180.0,
                away_prob: 0.9,
                expected_window: false,
                token: None,
                evidence: Evidence {
                    llr_time: 2.8,      // VERY suspicious time -> HIGH THREAT
                    llr_entry: 2.5,     // Forced entry -> HIGH THREAT (FIXED!)
                    llr_behavior: 3.2,  // Aggressive behavior -> HIGH THREAT
                    llr_identity: 2.1,  // Unknown people -> HIGH THREAT (FIXED!)
                    llr_presence: 2.0,  // Strong presence -> THREAT
                    llr_token: 2.5,     // No authorization -> HIGH THREAT (FIXED!)
                },
            }
        ),
        (
            "⚠️ ELEVATED: Unknown Person with Lock Picking Tools", 
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "suspicious_unknown".to_string(),
                rang_doorbell: false,
                knocked: false,
                dwell_s: 120.0,
                away_prob: 0.7,
                expected_window: false,
                token: None,
                evidence: Evidence {
                    llr_time: 1.5,      // Moderately suspicious time -> THREAT
                    llr_entry: 1.8,     // Suspicious entry -> THREAT (FIXED!)
                    llr_behavior: 2.1,  // Suspicious behavior -> THREAT
                    llr_identity: 1.5,  // Unknown person -> MODERATE THREAT (FIXED!)
                    llr_presence: 1.2,  // Clear presence -> THREAT
                    llr_token: 2.0,     // No authorization -> THREAT (FIXED!)
                },
            }
        ),
        (
            "📢 STANDARD: Unknown Person at Unusual Hour",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "unknown_visitor".to_string(),
                rang_doorbell: true,
                knocked: false,
                dwell_s: 45.0,
                away_prob: 0.5,
                expected_window: false,
                token: None,
                evidence: Evidence {
                    llr_time: 0.3,      // Somewhat unusual time -> MILD THREAT
                    llr_entry: -0.8,    // Normal entry (rang doorbell) -> REDUCES THREAT (FIXED!)
                    llr_behavior: -0.5, // Mostly normal -> REDUCES THREAT
                    llr_identity: 0.4,  // Unknown but polite -> MILD THREAT (FIXED!)
                    llr_presence: -0.2,  // Normal presence -> NEUTRAL
                    llr_token: 0.5,     // No token but rang doorbell -> MILD THREAT (FIXED!)
                },
            }
        ),
        (
            "⏳ WAIT: Possible Delivery Person",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "delivery_person".to_string(),
                rang_doorbell: true,
                knocked: true,
                dwell_s: 30.0,
                away_prob: 0.3,
                expected_window: true,
                token: None,
                evidence: Evidence {
                    llr_time: -0.5,     // Good delivery time -> REDUCES THREAT
                    llr_entry: -1.0,    // Professional entry -> REDUCES THREAT (FIXED!)
                    llr_behavior: -0.8, // Professional behavior -> REDUCES THREAT  
                    llr_identity: -0.2, // Uniform visible -> REDUCES THREAT (FIXED!)
                    llr_presence: -0.3, // Brief appropriate presence -> REDUCES THREAT
                    llr_token: 0.3,     // Expected but no token -> SLIGHT THREAT (FIXED!)
                },
            }
        ),
        (
            "😴 IGNORE: Recognized Family Member",
            Event {
                ts: Utc::now().timestamp() as f64,
                cam: "front_door_camera".to_string(),
                person_track: "family_member".to_string(),
                rang_doorbell: false,
                knocked: false,
                dwell_s: 10.0,
                away_prob: 0.1,
                expected_window: true,
                token: Some("family_key".to_string()),
                evidence: Evidence {
                    llr_time: -0.2,     // Normal time -> REDUCES THREAT
                    llr_entry: -1.5,    // Legitimate entry -> REDUCES THREAT (FIXED!)
                    llr_behavior: -1.2, // Normal family behavior -> REDUCES THREAT
                    llr_identity: -1.8, // RECOGNIZED family -> STRONGLY REDUCES THREAT (FIXED!)
                    llr_presence: -0.4, // Quick normal entry -> REDUCES THREAT
                    llr_token: -1.2,    // HAS authorization -> STRONGLY REDUCES THREAT (FIXED!)
                },
            }
        ),
    ];
    
    for (i, (description, event)) in scenarios.into_iter().enumerate() {
        println!("\n{}", "=".repeat(80));
        println!("🧪 TEST {}: {}", i + 1, description);
        println!("{}", "=".repeat(80));
        
        if let Some(result) = thinking_ai.process_event("test_home", event) {
            println!("🎯 THREAT PROBABILITY: {:.2}%", result.calibrated_probability * 100.0);
            println!("⚡ ALERT DECISION: {:?}", result.alert_decision);
            
            // Show evidence breakdown
            println!("\n📊 EVIDENCE BREAKDOWN:");
            let evidence = &result.fused_evidence;
            println!("   Time: {:+.2}", evidence.llr_time);
            println!("   Entry: {:+.2}", evidence.llr_entry);
            println!("   Behavior: {:+.2}", evidence.llr_behavior);
            println!("   Identity: {:+.2}", evidence.llr_identity);
            println!("   Presence: {:+.2}", evidence.llr_presence);
            println!("   Token: {:+.2}", evidence.llr_token);
            println!("   Total: {:+.2}", evidence.sum());
            
            println!("\n🧠 AI REASONING:");
            println!("{}", "─".repeat(100));
            println!("{}", thinking_ai.format_thinking_block(&result));
            println!("{}", "─".repeat(100));
            
        } else {
            println!("❌ NO AI ANALYSIS GENERATED");
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }
    
    println!("\n{}", "=".repeat(80));
    println!("🎯 FIXED EVIDENCE TEST COMPLETE!");
    println!("✅ EVIDENCE SEMANTICS: CORRECTED");
    println!("✅ THREAT ASSESSMENT: SHOULD BE ACCURATE NOW");
    println!("{}", "=".repeat(80));
}
