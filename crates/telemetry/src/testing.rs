/// Initializes telemetry integration for libtest environments.
pub fn init_test_telemetry() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if let Err(err) = tracing_subscriber::fmt().with_test_writer().try_init() {
            eprintln!("init_test_telemetry failed to init global tracing_subscriber: {err:?}");
        }
    });
}
