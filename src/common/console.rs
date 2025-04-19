use crate::arch::console::DebugConsole;
use core::fmt::Write;

#[macro_export]
macro_rules! println {
    () => {
        $crate::common::console::_print(format_args!("\n"))
    };
    ($fmt:expr $(, $($arg:tt)+)?) => {
        $crate::common::console::_print(format_args!("{}\n", format_args!($fmt $(, $($arg)+)?)))
    };
}

#[inline]
pub(crate) fn _print(args: core::fmt::Arguments) {
    DebugConsole.write_fmt(args).expect("can't print arguments");
}

// Write string through DebugConsole
impl Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|x| Self::putchar(*x));
        Ok(())
    }
}

#[cfg(feature = "log")]
impl log::Log for DebugConsole {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        use log::Level;

        let file = record.module_path();
        let line = record.line();
        let color_code = match record.level() {
            Level::Error => 31u8, // Red
            Level::Warn => 93,    // BrightYellow
            Level::Info => 34,    // Blue
            Level::Debug => 32,   // Green
            Level::Trace => 90,   // BrightBlack
        };
        println!(
            "\u{1B}[{}m\
                    [{}] <{}:{}> {}\
                    \u{1B}[0m",
            color_code,
            record.level(),
            file.unwrap(),
            line.unwrap(),
            record.args()
        );
    }

    fn flush(&self) {}
}

#[cfg(feature = "log")]
impl DebugConsole {
    pub(crate) fn log_init() {
        use log::LevelFilter;
        log::set_logger(&DebugConsole).unwrap();
        log::set_max_level(match option_env!("LOG") {
            Some("error") => LevelFilter::Error,
            Some("warn") => LevelFilter::Warn,
            Some("info") => LevelFilter::Info,
            Some("debug") => LevelFilter::Debug,
            Some("trace") => LevelFilter::Trace,
            _ => LevelFilter::Debug,
        });
    }
}
