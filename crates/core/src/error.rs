use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("klasör bulunamadı: {0}")]
    NotFound(PathBuf),
    #[error("bu bir Minecraft sunucu klasörüne benzemiyor")]
    NotAServer,
    #[error("sunucu çalışıyor, önce sunucuyu kapat")]
    ServerRunning,
    #[error("korunan dosyaya yazılamaz: {0}")]
    Protected(String),
    #[error("{file} yazıldıktan sonra doğrulanamadı, değişiklik geri alındı")]
    Invalid { file: String },
    #[error("yedek bulunamadı: {0}")]
    BackupMissing(String),
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    #[error("yedek kaydı okunamadı: {0}")]
    Manifest(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) trait IoContext<T> {
    fn ctx(self, context: impl FnOnce() -> String) -> Result<T>;
}

impl<T> IoContext<T> for std::io::Result<T> {
    fn ctx(self, context: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|source| Error::Io {
            context: context(),
            source,
        })
    }
}
