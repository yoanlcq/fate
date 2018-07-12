use std::time::Duration;
use std::collections::VecDeque;
use frame_time::FrameTimeManager;
use message::Message;
use scene::Scene;
use input::Input;
use fate::vek::Extent2;


#[derive(Debug)]
pub struct SharedGame {
    pub t: Duration, // Total physics time since the game started (accumulation of per-tick delta times)
    pub frame_time_manager: FrameTimeManager,
    pub pending_messages: VecDeque<Message>,
    pub scene: Scene,
    pub input: Input,
}

pub type G = SharedGame;


impl SharedGame {
    pub fn new(canvas_size: Extent2<u32>) -> Self {
        Self {
            t: Duration::default(),
            frame_time_manager: FrameTimeManager::with_max_len(60),
            pending_messages: VecDeque::new(),
            scene: Scene::new(),
            input: Input::new(canvas_size),
        }
    }
    #[allow(dead_code)]
    pub fn push_message(&mut self, msg: Message) {
        self.pending_messages.push_back(msg);
    }
}


