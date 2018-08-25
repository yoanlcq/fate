use std::sync::{Mutex, atomic::{self, AtomicBool, AtomicUsize}};
use std::path::Path;
use std::fs::File;
use std::io::{self, Read};
use std::mem;
use {Task};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FileProgress {
    pub done: bool,
    pub nread: usize,
    pub nsize: usize,
}

#[derive(Debug, Default)]
struct Data {
    file: Option<File>,
    buf: Vec<u8>,
    err: Option<io::Error>,
    nread: usize,
    nsize: usize,
}

#[derive(Debug)]
pub struct ReadFile {
    path: Box<Path>,
    done: AtomicBool,
    nread: AtomicUsize,
    nsize: AtomicUsize,
    data: Mutex<Data>,
}

impl ReadFile {
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        Self {
            path: Box::from(p.as_ref()),
            done: AtomicBool::new(false),
            nread: AtomicUsize::new(0),
            nsize: AtomicUsize::new(0),
            data: Mutex::new(Data::default()),
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Task for ReadFile {
    type Progress = FileProgress;
    type Result = io::Result<Vec<u8>>;
    fn resume(&self) {
        if self.is_complete() {
            return;
        }

        let mut data = self.data.lock().unwrap();
        if data.file.is_none() {
            match File::open(&self.path) {
                Ok(file) => match file.metadata() {
                    Ok(m) => {
                        let nsize = m.len();
                        assert!(nsize <= ::std::usize::MAX as u64);
                        data.nsize = nsize as usize;
                        data.file = Some(file);
                        data.buf = Vec::<u8>::with_capacity(data.nsize);
                        self.nsize.store(data.nsize, atomic::Ordering::SeqCst);
                    },
                    Err(err) => {
                        data.err = Some(err);
                        return self.done.store(true, atomic::Ordering::SeqCst);
                    },
                },
                Err(err) => {
                    data.err = Some(err);
                    return self.done.store(true, atomic::Ordering::SeqCst);
                },
            }
        }

        assert!(data.err.is_none());
        assert!(data.file.is_some());

        // Damn borrow checker :(
        let nb_to_read = ::std::cmp::min(4096, data.nsize - data.nread);
        let buf_range = data.nread .. data.nread + nb_to_read;

        // Acquiring the lock after result() was called by another thread
        if data.buf.capacity() == 0 {
            return; 
        }

        assert!(data.buf.capacity() >= buf_range.end);
        unsafe {
            data.buf.set_len(buf_range.end);
        }
        let mut file = data.file.take().unwrap();
        let ret = file.read(&mut data.buf[buf_range]);
        data.file = Some(file);
        match ret {
            Err(err) => {
                data.err = Some(err);
                return self.done.store(true, atomic::Ordering::SeqCst);
            },
            Ok(nret) => {
                data.nread += nret;
                assert!(data.nread <= data.nsize);
                unsafe {
                    let nread = data.nread;
                    data.buf.set_len(nread);
                }
                self.nread.store(data.nread, atomic::Ordering::SeqCst);
                if data.nread == data.nsize {
                    return self.done.store(true, atomic::Ordering::SeqCst);
                }
            },
        }
    }
    fn is_complete(&self) -> bool {
        self.done.load(atomic::Ordering::SeqCst)
    }
    fn progress(&self) -> Self::Progress {
        Self::Progress {
            done: self.done.load(atomic::Ordering::SeqCst),
            nread: self.nread.load(atomic::Ordering::SeqCst),
            nsize: self.nsize.load(atomic::Ordering::SeqCst),
        }
    }
    fn result(&self) -> Self::Result {
        debug_assert!(self.is_complete());
        let mut data = self.data.lock().unwrap();
        if let Some(err) = data.err.take() {
            return Err(err);
        }
        Ok(mem::replace(&mut data.buf, vec![]))
    }
}

