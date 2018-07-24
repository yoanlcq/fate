/*
NOTE: Je fais mon propre jeu ! Je choisis et je vois si ça marche ou pas.
Archi:
- 1 thread pool (NOTE: Le but des masks est de s'assurer que pas tout les threads ne se ruent sur l'IO)
  - Thread 1: DiskIO | NetIO
  - Thread 2: DiskIO
  - Thread 3: Anim | Physics | AI
  - ....
- N job queues
  - DiskIO queue;
  - NetIO queue;
  - AI queue;
  - Physics queue;
  - Anim queue;
  - ......
  - Chaque thread loope en pollant les queues auxquelles il a accès;
- Abstraction pour les futurs
  - Rationale: Adaptée au système. Simple. Portable (aux consoles, plus tard).
  - Utilisée pour n'importe quoi qui est async; c'est aussi avec ça que le moteur implémente les tasks.
    Il suffit de faire `let future_stuff = do_stuff_async()`, puis plus tard `future_stuff.wait()`.
    Je veux que `wait()` garde le thread occupé à processer d'autres tasks.
    Je veux aussi limiter un maximum les appels à lock() (i.e les Mutex en général).
- Managers :
  - ResourceLoader
  - Physics
    - Peut donner des AsyncRaycastHit.
*/
// FIXME:
// - Faire pareil pour du raycasting; S'en servir pour extraire le code commun dans des traits.
// - Dormir quand il n'y a plus d'items dans les queues;
// - Faire du work stealing; Quand on wait, on cherche à prendre des éléments de la queue;
// - Est-ce qu'on peut leur faire implémenter Future et Stream ?
extern crate futures;

use std::thread;
use std::sync::{Arc, Mutex, atomic::{self, AtomicBool, AtomicUsize, AtomicIsize, Ordering}};
use std::collections::VecDeque;
use std::mem;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

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
#[derive(Debug)]
pub struct LoadingFile(Arc<LoadingFileTaskData>);

impl Game {
    pub fn load_file<P: AsRef<Path>>(&self, path: P) -> LoadingFile {
        let task_data = Arc::new(LoadingFileTaskData {
            thread_id: AtomicIsize::new(-1),
            nb_bytes_read: AtomicUsize::new(0),
            total_nb_bytes: AtomicUsize::new(0),
            file_path: Box::from(path.as_ref()),
            state: Mutex::new(LoadingFileTaskState::Pending),
        });
        self.job_queue.lock().unwrap().push_back(LoadingFile(task_data.clone()));
        LoadingFile(task_data)
    }
}
#[derive(Debug, Copy, Clone)]
pub enum LoadingFileProgress {
    Pending,
    Loading { nb_bytes_read: usize, total_nb_bytes: usize, thread_id: usize },
    Complete { total_nb_bytes: usize, thread_id: usize, },
}

#[derive(Debug, Default)]
pub struct Game {
    pub quit: AtomicBool,
    pub job_queue: Mutex<VecDeque<LoadingFile>>,
}
#[derive(Debug)]
pub struct ThreadContext {
    pub name: String,
    pub i: u32,
    pub g: Arc<Game>,
}

pub fn thread_proc(cx: ThreadContext) {
    while !cx.g.quit.load(Ordering::SeqCst) {
        let task = {
            let mut lock = cx.g.job_queue.lock().unwrap();
            lock.pop_front()
        };
        // FIXME: Sleep if there are no more tasks
        if let Some(task) = task {
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
                            task.0.thread_id.store(cx.i as _, Ordering::SeqCst); // NOTE: Care to do it _after_ setting total_nb_bytes
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
    }
}

pub trait Progress {
    fn is_complete(&self) -> bool;
}
pub trait Loading {
    type Item;
    type Error;
    type Progress: Progress;
    fn poll(&self) -> Self::Progress;
    fn wait(self) -> Result<Self::Item, Self::Error>;
    fn cancel(self); // NOTE: Should this return a Future indicating the progress of cancellation??
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
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Late<T: Loading> { // Convenience enum for something that may or may not be loaded.
    Loading(T),
    Complete(Result<T::Item, T::Error>),
}
impl<T: Loading> From<T> for Late<T> {
    fn from(loading: T) -> Self {
        Self::new(loading)
    }
}
impl<T: Loading> From<Result<T::Item, T::Error>> for Late<T> {
    fn from(result: Result<T::Item, T::Error>) -> Self {
        Late::Complete(result)
    }
}
impl<T: Loading> Late<T> {
    pub fn new(loading: T) -> Self { Late::Loading(loading) }
    pub fn is_loading(&self) -> bool { if let Late::Loading(_) = *self { true } else { false } }
    pub fn is_loaded(&self) -> bool { if let Late::Complete(Ok(_)) = *self { true } else { false } }
    pub fn is_failed(&self) -> bool { if let Late::Complete(Err(_)) = *self { true } else { false } }
    pub fn poll(&self) -> Option<T::Progress> {
        match *self {
            Late::Loading(ref loading) => Some(loading.poll()),
            Late::Complete(_) => None,
        }
    }
    pub fn wait(self) -> Result<T::Item, T::Error> {
        match self {
            Late::Loading(loading) => loading.wait(),
            Late::Complete(result) => result,
        }
    }
    pub fn get_mut(&mut self) -> Result<&mut T::Item, &mut T::Error> {
        let new = match *self {
            Late::Loading(ref loading) => Late::Complete(unsafe { ::std::ptr::read(loading) }.wait()),
            Late::Complete(ref result) => Late::Complete(unsafe { ::std::ptr::read(result) }),
        };
        mem::forget(mem::replace(self, new));
        match *self {
            Late::Complete(ref mut result) => result.as_mut(),
            _ => unreachable!(),
        }
    }
}

fn main() {
    let g = Arc::new(Game::default());
    let n_extra_threads = 2;
    let mut threads = Vec::new();
    for i in 1..(1+n_extra_threads) {
        let cx = ThreadContext {
            name: format!("Extra thread {}", i),
            i,
            g: g.clone(),
        };
        threads.push(thread::spawn(move || thread_proc(cx)));
    }

    let file_paths = [
        "huge.dat",
        "img.png",
        "Cargo.toml",
        "Cargo.lock",
    ];
    let files: Vec<_> = file_paths.iter().map(|s| g.load_file(&s)).collect();

    let mut all_complete = false;
    while !all_complete {
        all_complete = true;
        for (path, f) in file_paths.iter().zip(files.iter()) {
            let progress = f.poll();
            println!("Loading `{}`... ({:?})", path, progress);
            if !progress.is_complete() {
                all_complete = false;
            }
        }
        println!("--");
        println!("--");
    }
    for f in files {
        let _data = f.wait().unwrap();
    }

    g.quit.store(true, Ordering::SeqCst);
    for t in threads {
        t.join().unwrap();
    }
}
