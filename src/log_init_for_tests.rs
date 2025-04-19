#[cfg(test)]
#[ctor::ctor]
fn init_logging() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} [{l}] {M}[{L}] - {m}{n}")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
