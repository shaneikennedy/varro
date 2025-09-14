use anyhow::{Context, Result};
use log::info;
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

const MANIFEST_FILENAME: &str = "manifest.varro";

pub(crate) trait FileSystem: Send + Sync {
    fn read_from_index(&self, filename: &Path) -> Result<Vec<u8>>;
    fn read_from_documents(&self, filename: &Path) -> Result<Vec<u8>>;
    fn read_from_manifest(&self) -> Result<Vec<u8>>;
    fn write_to_index(&self, filename: &Path, contents: Vec<u8>) -> Result<()>;
    fn write_to_document(&self, filename: &Path, contents: Vec<u8>) -> Result<()>;
    fn write_to_manifest(&self, contents: Vec<u8>) -> Result<()>;
}

pub(crate) struct LocalFileSystem {
    /// Where on the filesystem to store files and their indexes
    index_path: PathBuf,

    /// Where the full document objects are actually stored
    documents_path: PathBuf,
}

impl LocalFileSystem {
    pub fn new(index_path: &Path) -> Result<Self> {
        let documents_path = index_path.join("documents");
        match index_path.exists() {
            true => info!("Index dir exists"),
            false => fs::create_dir(index_path)?,
        };
        match documents_path.exists() {
            true => info!("Documents subdir dir exists"),
            false => fs::create_dir(documents_path.clone())?,
        };
        Ok(Self {
            index_path: index_path.to_path_buf(),
            documents_path,
        })
    }

    fn read(&self, path: &PathBuf) -> Result<Vec<u8>> {
        let contents = fs::read(path).with_context(|| "unable to read file")?;
        Ok(contents)
    }

    fn write(&self, filename: &PathBuf, contents: Vec<u8>) -> Result<()> {
        fs::write(filename, contents).with_context(|| "failed to write contents")?;
        Ok(())
    }
}

impl FileSystem for LocalFileSystem {
    fn read_from_index(&self, filename: &Path) -> Result<Vec<u8>> {
        self.read(&self.index_path.join(filename))
    }

    fn write_to_index(&self, filename: &Path, contents: Vec<u8>) -> Result<()> {
        self.write(&self.index_path.join(filename), contents)
    }

    fn read_from_documents(&self, filename: &Path) -> Result<Vec<u8>> {
        self.read(&self.documents_path.join(filename))
    }

    fn write_to_document(&self, filename: &Path, contents: Vec<u8>) -> Result<()> {
        self.write(&self.documents_path.join(filename), contents)
    }

    fn read_from_manifest(&self) -> Result<Vec<u8>> {
        self.read_from_index(Path::new(MANIFEST_FILENAME))
    }

    fn write_to_manifest(&self, contents: Vec<u8>) -> Result<()> {
        self.write_to_index(Path::new(MANIFEST_FILENAME), contents)
    }
}
