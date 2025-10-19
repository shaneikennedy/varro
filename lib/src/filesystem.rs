use anyhow::{Context, Result};
use log::info;
#[cfg(feature = "s3")]
use s3::{Bucket, creds::Credentials};
use std::collections::HashSet;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const MANIFEST_FILENAME: &str = "manifest.varro";

pub(crate) trait FileSystem: Send + Sync {
    fn list_documents(&self) -> HashSet<String>;
    fn read_from_index(&self, filename: &Path) -> Result<Vec<u8>>;
    fn read_from_documents(&self, filename: &Path) -> Result<Vec<u8>>;
    fn read_from_manifest(&self) -> Result<Vec<u8>>;
    fn write_to_index(&self, filename: &Path, contents: Vec<u8>) -> Result<()>;
    fn write_to_document(&self, filename: &Path, contents: Vec<u8>) -> Result<()>;
    fn write_to_manifest(&self, contents: Vec<u8>) -> Result<()>;
    fn remove_from_index(&self, filename: &Path) -> Result<()>;
    fn remove_from_documents(&self, filename: &Path) -> Result<()>;
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
        fs::read(path).context("unable to read file")
    }

    fn write(&self, path: &PathBuf, contents: Vec<u8>) -> Result<()> {
        fs::write(path, contents).with_context(|| "failed to write contents")
    }

    fn delete(&self, path: &PathBuf) -> Result<()> {
        fs::remove_file(path).context("Unable to remove file")
    }

    fn list(&self) -> HashSet<String> {
        let docs = fs::read_dir(self.documents_path.clone()).unwrap();
        docs.filter_map(|f| f.ok())
            .map(|f| f.file_name().into_string().unwrap())
            .collect()
    }
}

impl FileSystem for LocalFileSystem {
    fn list_documents(&self) -> HashSet<String> {
        self.list()
    }
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

    fn remove_from_index(&self, filename: &Path) -> Result<()> {
        self.delete(&self.index_path.join(filename))
    }

    fn remove_from_documents(&self, filename: &Path) -> Result<()> {
        self.delete(&self.documents_path.join(filename))
    }
}

pub(crate) struct TempFileSystem {
    /// Where on the filesystem to store files and their indexes
    index_path: PathBuf,

    /// Where the full document objects are actually stored
    documents_path: PathBuf,

    /// Reference to the tempdir so that it only goes out
    /// of scope when the TempFileSystem does
    _temp_dir: TempDir,
}

impl TempFileSystem {
    pub fn new(path: Option<&Path>) -> Result<Self> {
        let temp_dir = tempfile::Builder::new().tempdir_in(path.unwrap_or(Path::new("./")))?;
        let path = temp_dir.path();
        let documents_path = path.join("documents");
        fs::create_dir(documents_path.clone())?;
        Ok(Self {
            index_path: path.to_path_buf(),
            documents_path,
            _temp_dir: temp_dir,
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

    fn delete(&self, path: &PathBuf) -> Result<()> {
        fs::remove_file(path).context("Unable to remove file")
    }

    #[allow(dead_code)]
    fn index_path(&self) -> PathBuf {
        self.index_path.clone()
    }

    fn list(&self) -> HashSet<String> {
        let docs = fs::read_dir(self.documents_path.clone()).unwrap();
        docs.filter_map(|f| f.ok())
            .map(|f| f.file_name().into_string().unwrap())
            .collect()
    }
}

impl FileSystem for TempFileSystem {
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

    fn remove_from_index(&self, filename: &Path) -> Result<()> {
        self.delete(&self.index_path.join(filename))
    }

    fn list_documents(&self) -> HashSet<String> {
        self.list()
    }

    fn remove_from_documents(&self, filename: &Path) -> Result<()> {
        self.delete(&self.documents_path.join(filename))
    }
}

#[cfg(feature = "s3")]
pub(crate) struct S3FileSystem {
    /// Where on the filesystem to store files and their indexes
    index_path: String,

    /// Where the full document objects are actually stored
    documents_path: String,

    bucket: Bucket,
}

#[cfg(feature = "s3")]
impl S3FileSystem {
    pub fn new(path: &Path) -> Result<Self> {
        let path = path.to_str().unwrap().to_string();
        let bucket = Bucket::new(
            path.as_str(),
            "eu-central-1".parse()?,
            // Credentials are collected from environment, config, profile or instance metadata
            Credentials::default()?,
        )?;
        Ok(Self {
            index_path: path.clone(),
            documents_path: path + "/documents",
            bucket: *bucket,
        })
    }

    fn read(&self, path: &str) -> Result<Vec<u8>> {
        let contents = self
            .bucket
            .get_object(path)
            .with_context(|| "unable to read file")?;
        Ok(contents.to_vec())
    }

    fn write(&self, filename: &str, contents: Vec<u8>) -> Result<()> {
        self.bucket
            .put_object(filename, &contents[..])
            .with_context(|| "failed to write contents")?;
        Ok(())
    }

    fn delete(&self, filename: &str) -> Result<()> {
        self.bucket
            .delete_object(filename)
            .context("failed to delete file")?;
        Ok(())
    }

    fn list(&self) -> HashSet<String> {
        let objects = self.bucket.list(self.documents_path.clone(), None).unwrap();
        objects.into_iter().map(|o| o.name).collect()
    }
}

#[cfg(feature = "s3")]
impl FileSystem for S3FileSystem {
    fn list_documents(&self) -> HashSet<String> {
        self.list()
    }

    fn read_from_index(&self, filename: &Path) -> Result<Vec<u8>> {
        self.read(format!("{}/{}", &self.index_path, filename.to_str().unwrap()).as_str())
    }

    fn write_to_index(&self, filename: &Path, contents: Vec<u8>) -> Result<()> {
        self.write(
            format!("{}/{}", &self.index_path, filename.to_str().unwrap()).as_str(),
            contents,
        )
    }

    fn read_from_documents(&self, filename: &Path) -> Result<Vec<u8>> {
        self.read(format!("{}/{}", &self.documents_path, filename.to_str().unwrap()).as_str())
    }

    fn write_to_document(&self, filename: &Path, contents: Vec<u8>) -> Result<()> {
        self.write(
            format!("{}/{}", &self.documents_path, filename.to_str().unwrap()).as_str(),
            contents,
        )
    }

    fn read_from_manifest(&self) -> Result<Vec<u8>> {
        self.read_from_index(Path::new(MANIFEST_FILENAME))
    }

    fn write_to_manifest(&self, contents: Vec<u8>) -> Result<()> {
        self.write_to_index(Path::new(MANIFEST_FILENAME), contents)
    }

    fn remove_from_index(&self, filename: &Path) -> Result<()> {
        self.delete(format!("{}/{}", self.index_path, filename.to_str().unwrap()).as_str())
    }

    fn remove_from_documents(&self, filename: &Path) -> Result<()> {
        self.delete(format!("{}/{}", self.documents_path, filename.to_str().unwrap()).as_str())
    }
}

#[cfg(test)]
mod filesystem_temp_tests {
    use super::*;

    #[test]
    fn test_is_temporary() {
        let fs = TempFileSystem::new(None).unwrap();
        let path = fs.index_path();
        assert!(Path::exists(&path));
        assert!(Path::exists(&path.join("documents")));
        drop(fs);
        assert!(!Path::exists(&path));
    }

    #[test]
    fn test_read_from_index() -> Result<()> {
        let fs = TempFileSystem::new(None).unwrap();
        let path = fs.index_path();
        let _ = fs.write_to_index(Path::new("file"), vec![1, 2, 3, 4, 5]);
        assert!(Path::exists(&path.join("file")));
        let contents = fs.read_from_index(&path.join("file"))?;
        assert_eq!(contents, vec![1, 2, 3, 4, 5]);
        Ok(())
    }

    #[test]
    fn test_remove_from_index() -> Result<()> {
        let fs = TempFileSystem::new(None).unwrap();
        let path = fs.index_path();
        let _ = fs.write_to_index(Path::new("file"), vec![1, 2, 3, 4, 5]);
        fs.remove_from_index(&path.join("file"))?;
        assert!(!Path::exists(&path.join("file")));
        Ok(())
    }
}
