use std::thread;
use std::sync::{Arc, Mutex, atomic::{self, AtomicBool, AtomicUsize, AtomicIsize, Ordering}};
use std::collections::VecDeque;
use std::mem;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use game::game::{MtShared, ThreadContext};
use super::{Progress, Loading};

#[derive(Debug)]
pub enum LoadingFileTaskState {
    Pending,
    Reading { file: File, buf: Vec<u8>, nb_bytes_read: usize, total_nb_bytes: usize, },
    Succeeded(Vec<u8>),
    Errored(String),
}

#[derive(Debug)]
pub struct LoadingFileTaskData {
    // Lightweight part, so poll() is super responsive
    pub thread_id: AtomicIsize,
    pub nb_bytes_read: AtomicUsize,
    pub total_nb_bytes: AtomicUsize,
    pub file_path: Box<Path>, // This used to be in LoadingFileTaskState::Pending so it was freed as soon as the file was opened, but I figured it may be useful to keep it for loggin/debugging. IDK.
    // Heavyweight part
    pub state: Mutex<LoadingFileTaskState>,
}

#[derive(Debug, Copy, Clone)]
pub enum LoadingFileProgress {
    Pending,
    Loading { nb_bytes_read: usize, total_nb_bytes: usize, thread_id: usize },
    Complete { total_nb_bytes: usize, thread_id: usize, },
}

#[derive(Debug)]
pub struct LoadingFile(Arc<LoadingFileTaskData>);

impl MtShared {
    pub fn load_file<P: AsRef<Path>>(&self, path: P) -> LoadingFile {
        let task_data = Arc::new(LoadingFileTaskData {
            thread_id: AtomicIsize::new(-1),
            nb_bytes_read: AtomicUsize::new(0),
            total_nb_bytes: AtomicUsize::new(0),
            file_path: Box::from(path.as_ref()),
            state: Mutex::new(LoadingFileTaskState::Pending),
        });
        self.file_io_tasks_queue.lock().unwrap().push_back(LoadingFile(task_data.clone()));
        LoadingFile(task_data)
    }
}

impl Progress for LoadingFileProgress {
    fn is_complete(&self) -> bool {
        match *self {
            LoadingFileProgress::Complete { .. } => true,
            _ => false,
        }
    }
}
impl Loading for LoadingFile {
    type Item = Vec<u8>;
    type Error = String;
    type Progress = LoadingFileProgress;
    fn poll(&self) -> LoadingFileProgress {
        let thread_id = self.0.thread_id.load(Ordering::SeqCst);
        if thread_id < 0 {
            LoadingFileProgress::Pending
        } else {
            assert!(thread_id >= 0);
            let thread_id = thread_id as usize;
            let total_nb_bytes = self.0.total_nb_bytes.load(Ordering::SeqCst);
            let nb_bytes_read = self.0.nb_bytes_read.load(Ordering::SeqCst);
            if nb_bytes_read == total_nb_bytes {
                LoadingFileProgress::Complete { total_nb_bytes, thread_id }
            } else {
                assert!(nb_bytes_read < total_nb_bytes);
                LoadingFileProgress::Loading { nb_bytes_read, total_nb_bytes, thread_id }
            }
        }
    }
    fn wait(mut self) -> Result<Vec<u8>, String> {
        loop {
            match Arc::try_unwrap(self.0) {
                Err(arc) => self.0 = arc,
                Ok(data) => {
                    let state = data.state.into_inner().unwrap();
                    return match state {
                        LoadingFileTaskState::Succeeded(data) => Ok(data),
                        LoadingFileTaskState::Errored(e) => Err(e),
                        _ => unreachable!(),
                    };
                },
            }
            atomic::spin_loop_hint();
        }
    }
    fn cancel(self) { drop(self) }
}

pub fn process_file_io_task(cx: &ThreadContext, task: LoadingFile) {
    while Arc::strong_count(&task.0) > 1 { // If the client is not interested anymore, we might as well not do it
        // Make it progress a bit !
        let mut state = task.0.state.lock().unwrap();
        let next_state = match *state {
            LoadingFileTaskState::Succeeded(_) => break,
            LoadingFileTaskState::Errored(_) => break,
            LoadingFileTaskState::Pending => match File::open(&task.0.file_path) {
                Err(e) => Some(LoadingFileTaskState::Errored(format!("{}", e))),
                Ok(mut file) => {
                    let total_nb_bytes = {
                        let total = file.seek(SeekFrom::End(0)).unwrap();
                        file.seek(SeekFrom::Start(0)).unwrap();
                        assert!(total <= ::std::usize::MAX as u64);
                        total as usize
                    };
                    let mut buf = Vec::with_capacity(total_nb_bytes);
                    unsafe {
                        buf.set_len(total_nb_bytes);
                    }
                    task.0.total_nb_bytes.store(total_nb_bytes, Ordering::SeqCst);
                    task.0.thread_id.store(cx.id.i as _, Ordering::SeqCst); // NOTE: Care to do it _after_ setting total_nb_bytes
                    Some(LoadingFileTaskState::Reading { file, buf, nb_bytes_read: 0, total_nb_bytes })
                },
            },
            LoadingFileTaskState::Reading { ref mut file, ref mut buf, ref mut nb_bytes_read, total_nb_bytes } => {
                let nb_to_read = ::std::cmp::min(4096, total_nb_bytes - *nb_bytes_read);
                match file.read(&mut buf[*nb_bytes_read .. *nb_bytes_read + nb_to_read]) {
                    Err(e) => Some(LoadingFileTaskState::Errored(format!("{}", e))),
                    Ok(nbytes) => {
                        *nb_bytes_read += nbytes;
                        assert!(*nb_bytes_read <= total_nb_bytes);
                        task.0.nb_bytes_read.store(*nb_bytes_read, Ordering::SeqCst);
                        if *nb_bytes_read == total_nb_bytes {
                            Some(LoadingFileTaskState::Succeeded(mem::replace(buf, Vec::new())))
                        } else {
                            None
                        }
                    },
                }
            },
        };
        if let Some(next_state) = next_state {
            *state = next_state;
        }
        atomic::spin_loop_hint();
    }
}
