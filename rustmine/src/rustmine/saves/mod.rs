use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct Save {
    pub name: String,
}

impl Save {
    fn get_saves_dir() -> PathBuf {
        dirs::data_local_dir()
            .expect("Failed to get local data dir")
            .join("rustmine")
            .join("saves")
    }

    fn new(name: String) -> Save {
        Save { name }
    }

    pub fn create_dir() -> std::io::Result<()> {
        fs::create_dir_all(Self::get_saves_dir())
    }

    pub fn list() -> Vec<Save> {
        fs::read_dir(Self::get_saves_dir())
            .expect("Failed to list saves")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok() && entry.file_type().unwrap().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .map(Self::new)
            .collect()
    }

    pub fn get_by_name(name: &str) -> Save {
        Self::new(name.to_string())
    }

    fn get_save_dir(&self) -> PathBuf {
        Self::get_saves_dir().join(&self.name)
    }

    pub fn create(&self) -> std::io::Result<&Self> {
        let dir = self.get_save_dir();
        log::info!("Creating saves dir: {}", dir.display());
        fs::create_dir_all(dir)?;
        Ok(self)
    }

    pub fn delete(&self) -> std::io::Result<()> {
        log::info!("Deleting saves dir: {}", self.get_save_dir().display());
        fs::remove_dir_all(self.get_save_dir())
    }

    pub fn get_chunks_file(&self) -> PathBuf {
        self.get_save_dir().join("chunks.dat")
    }

    pub fn get_world_gen_settings_file(&self) -> PathBuf {
        self.get_save_dir().join("world_gen_settings.dat")
    }

    pub fn get_world_metadata_file(&self) -> PathBuf {
        self.get_save_dir().join("world_metadata.dat")
    }

    pub fn get_camera_file(&self) -> PathBuf {
        self.get_save_dir().join("camera.dat")
    }
}
