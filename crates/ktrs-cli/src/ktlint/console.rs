//! The process streams the ktlint CLI writes to, swappable for in-process tests.

use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::sync::{Arc, Mutex};

/// Java's `println` (and logback's `%n`) end lines with the platform separator.
pub const LINE_SEPARATOR: &str = if cfg!(windows) { "\r\n" } else { "\n" };

type Stream = Arc<Mutex<Box<dyn Write + Send>>>;

/// `System.in`, `System.out` and `System.err`.
#[derive(Clone)]
pub struct Console {
    input: Arc<Mutex<Box<dyn Read + Send>>>,
    out: Stream,
    err: Stream,
    process_streams: bool,
}

/// The bytes a captured [`Console`] received.
#[derive(Clone, Default)]
pub struct Captured(Arc<Mutex<Vec<u8>>>);

impl Captured {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

impl Write for Captured {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Console {
    pub fn std() -> Console {
        Console { process_streams: true, ..Console::new(Box::new(io::stdin()), Box::new(io::stdout()), Box::new(io::stderr())) }
    }

    pub fn new(input: Box<dyn Read + Send>, out: Box<dyn Write + Send>, err: Box<dyn Write + Send>) -> Console {
        let (input, out, err) = (Arc::new(Mutex::new(input)), Arc::new(Mutex::new(out)), Arc::new(Mutex::new(err)));
        Console { input, out, err, process_streams: false }
    }

    /// Whether this is the process's own stdin/stdout/stderr, which a child process can inherit.
    pub fn is_process_streams(&self) -> bool {
        self.process_streams
    }

    /// A console reading `stdin` and capturing stdout and stderr.
    pub fn capture(stdin: &[u8]) -> (Console, Captured, Captured) {
        let (out, err) = (Captured::default(), Captured::default());
        let console = Console::new(Box::new(io::Cursor::new(stdin.to_vec())), Box::new(out.clone()), Box::new(err.clone()));
        (console, out, err)
    }

    /// `System.in.readBytes()`.
    pub fn read_stdin(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let _ = self.input.lock().unwrap().read_to_end(&mut bytes);
        bytes
    }

    pub fn out(&self, text: &str) {
        write_flush(&self.out, text);
    }

    pub fn err(&self, text: &str) {
        write_flush(&self.err, text);
    }

    pub fn println_out(&self, line: &str) {
        self.out(&format!("{line}{LINE_SEPARATOR}"));
    }

    pub fn println_err(&self, line: &str) {
        self.err(&format!("{line}{LINE_SEPARATOR}"));
    }
}

fn write_flush(stream: &Stream, text: &str) {
    let mut stream = stream.lock().unwrap();
    let _ = stream.write_all(text.as_bytes()).and_then(|()| stream.flush());
}

/// Where a reporter's `PrintStream` goes.
pub enum Sink {
    Out(Console),
    Err(Console),
    File(BufWriter<File>),
    Buffer(Captured),
}

/// A `PrintStream` over a [`Sink`].
pub struct Printer {
    sink: Sink,
}

impl Printer {
    pub fn new(sink: Sink) -> Printer {
        Printer { sink }
    }

    /// A printer into a buffer, for tests.
    pub fn buffer() -> (Printer, Captured) {
        let captured = Captured::default();
        (Printer::new(Sink::Buffer(captured.clone())), captured)
    }

    pub fn print(&mut self, text: &str) {
        match &mut self.sink {
            Sink::Out(console) => console.out(text),
            Sink::Err(console) => console.err(text),
            Sink::File(file) => {
                let _ = file.write_all(text.as_bytes());
            }
            Sink::Buffer(captured) => {
                let _ = captured.write_all(text.as_bytes());
            }
        }
    }

    pub fn println(&mut self, line: &str) {
        self.print(&format!("{line}{LINE_SEPARATOR}"));
    }

    pub fn close(&mut self) {
        if let Sink::File(file) = &mut self.sink {
            let _ = file.flush();
        }
    }
}
