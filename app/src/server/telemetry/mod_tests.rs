use virtual_fs::VirtualFS;

use super::*;

// Tests that events are drained without writing a Rudder persistence file.
#[test]
fn test_persist_events_drains_without_writing_file() {
    let telemetry_api = TelemetryApi::new();

    VirtualFS::test(
        "test_persist_events_drains_without_writing_file",
        |dirs, _sandbox| {
            let user_id = Some("user".into());
            let anonymous_id = "anonymous_id".to_owned();

            warpui::telemetry::record_event(
                user_id.clone(),
                anonymous_id.clone(),
                "non UGC event name".into(),
                None,  /* payload */
                false, /* contains_ugc  */
                warpui::time::get_current_time(),
            );

            warpui::telemetry::record_event(
                user_id.clone(),
                anonymous_id.clone(),
                "UGC event name".into(),
                None, /* payload */
                true, /* contains_ugc  */
                warpui::time::get_current_time(),
            );

            let file_path = dirs.root().join("rudderstack");

            telemetry_api
                .flush_and_persist_events_at_path(10, PrivacySettingsSnapshot::mock(), &file_path)
                .expect("Should be able to drain events");

            assert!(!file_path.exists());
            assert!(warpui::telemetry::flush_events().is_empty());
        },
    );
}
