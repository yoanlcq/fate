use std::time::Duration;
use std::collections::VecDeque;
use frame_time::FrameTimeManager;
use message::Message;
use scene::Scene;
use input::Input;
use fate::math::Extent2;


#[derive(Debug)]
pub struct SharedGame {
    pub t: Duration, // Total physics time since the game started (accumulation of per-tick delta times)
    pub frame_time_manager: FrameTimeManager,
    pub pending_messages: VecDeque<Message>,
    pub scene: Scene,
    pub input: Input,
    pub res: Resources,
}

pub type G = SharedGame;


impl SharedGame {
    pub fn new(canvas_size: Extent2<u32>) -> Self {
        Self {
            t: Duration::default(),
            frame_time_manager: FrameTimeManager::with_max_len(60),
            pending_messages: VecDeque::new(),
            scene: Scene::new(canvas_size),
            input: Input::new(canvas_size),
            res: Resources::new().unwrap(),
        }
    }
    #[allow(dead_code)]
    pub fn push_message(&mut self, msg: Message) {
        self.pending_messages.push_back(msg);
    }
}

use std::env;
use std::path::{PathBuf, Path};
use fate::font::{Font, FontLoader};
use fate::img;
use atlas::Atlas;

#[derive(Debug)]
pub struct Resources {
    font_loader: FontLoader,
    data_path: PathBuf,
    basis33: Font,
    basis33_atlas: Atlas,
}

impl Resources {
    pub fn new() -> Result<Self, String> {
        let data_path = {
            let mut dir = env::current_exe().map_err(|io| format!("{}", io))?;
            loop {
                if !dir.pop() {
                    break Err(format!("Could not find `data` directory!"));
                }
                trace!("Searching for data path in `{}`", dir.display());
                dir.push("data");
                if dir.is_dir() {
                    break Ok(dir);
                }
                dir.pop();
            }
        }?;
        trace!("Found data path at `{}`", data_path.display());
        let font_loader = FontLoader::new().map_err(|e| format!("Could not create FontLoader: {}", e))?;
        let mut basis33 = font_loader.load_font(data_path.join(PathBuf::from("fonts/basis33/basis33.ttf"))).map_err(|e| format!("Could not load basis33 font: {}", e))?;
        basis33.set_height_px(16).unwrap();
        let basis33_atlas = Atlas::load(&basis33, &Atlas::all_supported_chars(), 128);

        if false { // Save the atlas so we can check it's fine?
            let path = data_path.join(PathBuf::from("fonts/basis33/atlas.png"));
            img::save_gray_u8(&path, img::ImageFormat::PNG, basis33_atlas.atlas.as_ref()).unwrap();
            info!("Saved `{}`", path.display());
        }

        Ok(Self {
            data_path,
            font_loader,
            basis33,
            basis33_atlas,
        })
    }
    pub fn data_path(&self) -> &Path {
        &self.data_path
    }
    pub fn font_loader(&self) -> &FontLoader {
        &self.font_loader
    }
}