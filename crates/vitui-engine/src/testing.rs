//! Sinks the engine's own tests write into.
//!
//! A frame's bytes are the only thing `present` produces, so every gate in ticket 03 is a statement
//! about what reached one of these.

use std::io::{Error, ErrorKind, Result, Write};
use std::sync::{Arc, Mutex};

/// What a [`Recorder`] saw.
#[derive(Default, Debug)]
pub(crate) struct Recording {
    /// Every byte, in order, reassembled across partial writes.
    pub(crate) bytes: Vec<u8>,
    /// How many `write` calls took at least one byte.
    pub(crate) writes: usize,
    /// How many `write` calls returned `WouldBlock`.
    pub(crate) retries: usize,
    calls: usize,
}

/// A sink that counts what it is given, and can be made as awkward as a real pipe.
#[derive(Clone, Default)]
pub(crate) struct Recorder {
    shared: Arc<Mutex<Recording>>,
    /// The most bytes one `write` will take.
    chunk: Option<usize>,
    /// Every Nth call returns `WouldBlock` instead of taking anything.
    would_block_every: Option<usize>,
}

impl Recorder {
    pub(crate) fn new() -> Recorder {
        Recorder::default()
    }

    /// A sink that takes `chunk` bytes at a time and returns `WouldBlock` every `block`th call.
    ///
    /// That is spec §8's honest way to test the partial-write loop: whether a real pipe fragments a
    /// write is the kernel's business, so the fragmentation is made deterministic instead.
    pub(crate) fn awkward(chunk: usize, block: usize) -> Recorder {
        Recorder {
            shared: Arc::new(Mutex::new(Recording::default())),
            chunk: Some(chunk),
            would_block_every: Some(block),
        }
    }

    /// A handle to the same recording, so a test can read what the engine wrote.
    pub(crate) fn handle(&self) -> Arc<Mutex<Recording>> {
        Arc::clone(&self.shared)
    }
}

impl Write for Recorder {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut r = self.shared.lock().expect("the recorder is never poisoned");
        r.calls += 1;
        if let Some(n) = self.would_block_every {
            if r.calls % n == 0 {
                r.retries += 1;
                return Err(Error::new(
                    ErrorKind::WouldBlock,
                    "the sink is being awkward",
                ));
            }
        }
        let take = self.chunk.map_or(buf.len(), |c| c.min(buf.len()));
        r.bytes.extend_from_slice(&buf[..take]);
        r.writes += 1;
        Ok(take)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
