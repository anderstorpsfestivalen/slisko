// Exercise the platform-neutral recovery and health logic with the host test
// runner. The firmware binary itself is cross-compiled for Xtensa and has its
// Rust test harness disabled.
#[path = "../../firmware/src/health.rs"]
mod health;
#[path = "../../firmware/src/recovery.rs"]
mod recovery;

#[test]
fn shared_health_handle_survives_state_transitions() {
    let health = health::Health::new(100);
    health.update(|snapshot| {
        snapshot.http = health::ServiceState::Retrying;
        snapshot.ddp = health::ServiceState::Stopped;
        snapshot.mdns = health::ServiceState::Retrying;
    });
    health.record_error("temporary outage");

    let snapshot = health.snapshot();
    assert_eq!(snapshot.http, health::ServiceState::Retrying);
    assert_eq!(snapshot.ddp, health::ServiceState::Stopped);
    assert_eq!(snapshot.mdns, health::ServiceState::Retrying);
    assert_eq!(snapshot.last_error.as_deref(), Some("temporary outage"));
}
