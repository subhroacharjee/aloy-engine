// log4rs
// log
// env_logger
pub fn init_logger() {
    use std::io::Write;
    let mut log_builder = env_logger::Builder::new();

    // default format is ugly
    // tz [LEVEL target] ...args
    log_builder
        .format_timestamp_secs()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Trace)
        .format(|buf, record| {
            let tz = buf.timestamp();
            writeln!(
                buf,
                "{} - [{} {}] -> {} ",
                tz,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}
