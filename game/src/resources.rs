use std::env;
use std::path::{PathBuf, Path};
use fate::font::{Font, FontLoader, Atlas};
use fate::img;

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
        let basis33_atlas = basis33.build_exhaustive_atlas(256);

        if env::var("export_font_atlases").is_ok() {
            let path = data_path.join(PathBuf::from("fonts/basis33/atlas.png"));
            img::save_gray_u8(&path, img::ImageFormat::PNG, basis33_atlas.img.as_ref()).unwrap();
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
