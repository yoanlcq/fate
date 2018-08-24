use std::time::Duration;
use std::collections::VecDeque;
use std::sync::Arc;
use frame_time::FrameTimeManager;
use message::Message;
use scene::Scene;
use input::Input;
use resources::Resources;
use async::{MtShared, AsyncFS};
use fate::math::Extent2;
use fate::lab::fps::FpsStats;

#[derive(Debug)]
pub struct SharedGame {
    pub t: Duration, // Total physics time since the game started (accumulation of per-tick delta times)
    pub frame_time_manager: FrameTimeManager,
    pub pending_messages: VecDeque<Message>,
    fps_stats_history: VecDeque<FpsStats>,
    pub mt: Arc<MtShared>,
    pub fs: AsyncFS,
    pub scene: Scene,
    pub input: Input,
    pub res: Resources,
}

pub type G = SharedGame;


impl SharedGame {
    pub fn new(canvas_size: Extent2<u32>, mt: Arc<MtShared>) -> Self {
        Self {
            t: Duration::default(),
            frame_time_manager: FrameTimeManager::with_max_len(60),
            pending_messages: VecDeque::new(),
            fps_stats_history: VecDeque::new(),
            fs: AsyncFS::new(mt.clone()),
            mt,
            scene: Scene::new(canvas_size),
            input: Input::new(canvas_size),
            res: Resources::new().unwrap(),
        }
    }
    #[allow(dead_code)]
    pub fn push_message(&mut self, msg: Message) {
        self.pending_messages.push_back(msg);
    }
    pub fn push_fps_stats(&mut self, fps_stats: FpsStats) {
        // Pretend we only keep 1 entry in the history
        self.fps_stats_history.pop_front();
        self.fps_stats_history.push_back(fps_stats);
    }
    pub fn last_fps_stats(&self) -> Option<FpsStats> {
        self.fps_stats_history.back().map(Clone::clone)
    }
}
