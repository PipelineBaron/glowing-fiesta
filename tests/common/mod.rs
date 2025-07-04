use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::sync::{Once, mpsc};

pub struct ChannelByteWriter {
    sender: mpsc::Sender<Option<Vec<u8>>>,
}

impl ChannelByteWriter {
    pub fn new(sender: mpsc::Sender<Option<Vec<u8>>>) -> Self {
        ChannelByteWriter { sender }
    }
}

impl Write for ChannelByteWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let copy = buf.to_vec();
        self.sender
            .send(Some(copy))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for ChannelByteWriter {
    fn drop(&mut self) {
        let _ = self.sender.send(None);
    }
}

pub struct ChannelByteReader {
    receiver: mpsc::Receiver<Option<Vec<u8>>>,
}

impl ChannelByteReader {
    pub fn new(receiver: mpsc::Receiver<Option<Vec<u8>>>) -> Self {
        ChannelByteReader { receiver }
    }

    pub fn read_to_string(&mut self) -> io::Result<String> {
        let mut result = String::new();
        while let Ok(Some(bytes)) = self.receiver.recv() {
            result.push_str(std::str::from_utf8(&bytes).unwrap());
        }
        Ok(result)
    }
}

thread_local! {
    pub static TEST_LOGS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}
static TEST_LOGS_INIT: Once = Once::new();

pub struct TestLogger;

static TEST_LOGGER: TestLogger = TestLogger;

impl TestLogger {
    pub fn reset() {
        TEST_LOGS_INIT.call_once(|| {
            let _ = log::set_logger(&TEST_LOGGER);
            log::set_max_level(log::LevelFilter::Info);
        });
        TEST_LOGS.with(|logs| {
            logs.borrow_mut().clear();
        });
    }
}

impl log::Log for TestLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        TEST_LOGS.with_borrow_mut(|logs| {
            let rendered_string = format!("{}", record.args());
            logs.push(rendered_string);
        });
    }

    fn flush(&self) {}
}
