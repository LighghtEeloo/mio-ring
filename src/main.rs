use anyhow::anyhow;
use directories_next::ProjectDirs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

slint::include_modules!();

#[derive(Serialize, Deserialize)]
pub struct Conf {
    #[serde(default)]
    pub cache_size: Option<usize>,
}

impl Conf {
    pub fn new(_config_root: impl AsRef<Path>) -> Self {
        // Todo: load config from file
        Self { cache_size: None }
    }
    pub fn config_path(config_root: PathBuf) -> PathBuf {
        config_root.join("config.toml")
    }
}

pub struct Cache {
    pub config_root: PathBuf,
    pub cache_root: PathBuf,
    pub pic_width: f32,
    pub pic_height: f32,
}

impl Cache {
    pub fn new(config_root: PathBuf, cache_root: PathBuf) -> Self {
        // Todo: recover cached state from file
        Self {
            config_root,
            cache_root,
            pic_width: 0.,
            pic_height: 0.,
        }
    }
    pub fn cache_path(cache_root: PathBuf) -> PathBuf {
        cache_root.join("cache.toml")
    }
}

pub struct Captured {
    pub image: screenshots::Image,
    pub time: u128,
}

impl Captured {
    pub fn path(&self, parent: impl Into<PathBuf>) -> PathBuf {
        parent.into().join(format!("{}.png", self.time))
    }
    pub fn write(&self, parent: impl Into<PathBuf>) -> anyhow::Result<PathBuf> {
        let buffer = self.image.to_png(None)?;
        let path = self.path(parent);
        fs::write(&path, buffer)?;

        Ok(path)
    }
}

#[derive(Debug)]
pub enum Committed {
    Full,
    Partial(Area2D),
}

pub struct App<Conf, Cache> {
    pub conf: Conf,
    pub cache: RefCell<Cache>,
}

impl App<Conf, Cache> {
    pub fn new() -> Self {
        let proj_dirs = ProjectDirs::from("", "LitiaEeloo", "MioRing")
            .expect("No valid project directory setup fomulated");
        let config_root = proj_dirs.config_dir().to_path_buf();
        let cache_root = proj_dirs.cache_dir().to_path_buf();
        for path in &[&config_root, &cache_root] {
            if !path.exists() {
                fs::create_dir_all(path).expect("Failed to create config directory");
            }
        }
        Self {
            conf: Conf::new(config_root.as_path()),
            cache: RefCell::new(Cache::new(config_root, cache_root)),
        }
    }
    pub fn capture(&self) -> anyhow::Result<Captured> {
        let screen = screenshots::Screen::all()?.into_iter().exactly_one()?;

        let image = screen.capture()?;
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();

        Ok(Captured { image, time })
    }
    pub fn launch(&self) -> anyhow::Result<(slint::Image, PathBuf)> {
        let captured = self.capture()?;
        self.cache.borrow_mut().pic_height = captured.image.height() as f32;
        self.cache.borrow_mut().pic_width = captured.image.width() as f32;
        let cache_parent = &self.cache.borrow().cache_root;

        let raw_path = captured.write(cache_parent)?;
        let img = slint::Image::load_from_path(raw_path.as_path())
            .map_err(|_| anyhow!("Failed to load image"))?;

        // Todo: cached?
        let cached = self.conf.cache_size.is_none();
        if !cached {
            std::fs::remove_file(captured.path(cache_parent))?;
        }
        Ok((img, raw_path))
    }
    pub fn commit(&self, committed: Committed, raw_path: impl AsRef<Path>) -> anyhow::Result<()> {
        println!("{:?}", committed);
        match committed {
            Committed::Full => {}
            Committed::Partial(area) => {
                let img = image::open(raw_path.as_ref())?;
                let img = img.crop_imm(
                    area.x as u32,
                    area.y as u32,
                    area.width as u32,
                    area.height as u32,
                );
                img.save(raw_path.as_ref())?;
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    // let tesseract_version = rusty_tesseract::get_tesseract_version()?;
    // println!("The tesseract version is: {:?}", tesseract_version);

    let app = Rc::new(App::new());
    let (raw_shot, raw_path) = app.launch()?;

    let win = MainWindow::new()?;
    let info = display_info::DisplayInfo::all()?
        .into_iter()
        .exactly_one()?;
    win.window()
        .set_position(slint::PhysicalPosition { x: 0, y: 0 });
    win.window().set_size(slint::PhysicalSize {
        width: info.width,
        height: info.height,
    });
    win.set_raw_shot(raw_shot);
    win.set_pic_width(app.cache.borrow().pic_width);
    win.set_pic_height(app.cache.borrow().pic_height);
    {
        let app_for_win = Rc::clone(&app);
        win.on_commit(move |is_full, area| {
            let committed = if is_full {
                Committed::Full
            } else {
                Committed::Partial(area)
            };
            app_for_win
                .commit(committed, raw_path.as_path())
                .expect("Failed to commit");
            std::process::exit(0)
        });
        win.on_exit(move || std::process::exit(0))
    }
    win.run()?;
    Ok(())
}
