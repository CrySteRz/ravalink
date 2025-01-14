use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

pub async fn loggers() {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Blue);

    let mut loggers = fern::Dispatch::new();

    loggers = loggers.format(move |out, message, record| {
        out.finish(format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            record.target(),
            colors.color(record.level()),
            message
        ))
    });

    loggers = loggers.level(log::LevelFilter::Debug);
    loggers = loggers
        .level_for("serenity", LevelFilter::Warn) 
        .level_for("tracing", LevelFilter::Warn);
    loggers = loggers.chain(std::io::stdout());

    loggers.apply().unwrap();
}
