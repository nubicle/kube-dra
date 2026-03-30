use std::path::PathBuf;

use tokio::net::UnixListener;

// Endpoint defines where and how to listen for incoming connections.
// A listener for a Unix domain socket gets created at the path.
// The listener always gets closed when shutting down.
pub(super) struct Endpoint {
    dir: PathBuf,
    file: String,
}

impl Endpoint {
    pub(super) fn new(dir: impl Into<PathBuf>, file: &str) -> Self {
        Endpoint {
            dir: dir.into(),
            file: file.to_string(),
        }
    }

    pub(super) fn path(&self) -> PathBuf {
        self.dir.join(&self.file)
    }

    pub(super) async fn listen(&self) -> anyhow::Result<UnixListener> {
        let socket_path = self.path();

        // remove stale sockets
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        Ok(UnixListener::bind(socket_path)?)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::FileTypeExt};

    use super::*;

    #[test]
    fn creates_correct_path() {
        let endpoint = Endpoint::new("/some/file", "path");
        let path = PathBuf::from("/some/file/path");
        assert_eq!(path, endpoint.path());
    }

    #[tokio::test]
    async fn removes_stale_socket_and_rebinds() {
        let tmp = std::env::temp_dir();
        let endpoint = Endpoint::new(&tmp, "test.sock");
        let _ = endpoint.listen().await;
        let meta_old = fs::metadata(endpoint.path()).unwrap();
        assert!(endpoint.path().exists());

        let endpoint = Endpoint::new(&tmp, "test.sock");
        let _ = endpoint.listen().await;
        assert!(endpoint.path().exists());

        let meta_new = fs::metadata(endpoint.path()).unwrap();
        assert!(meta_new.created().unwrap() > meta_old.created().unwrap());

        let file_type = meta_new.file_type();
        assert!(file_type.is_socket());
    }
}
