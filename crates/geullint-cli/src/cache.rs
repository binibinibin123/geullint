use anyhow::{Context, Result};
use geullint_core::Diagnostic;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_VERSION: u8 = 1;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CacheFile {
    pub version: u8,
    pub files: BTreeMap<String, CacheEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheEntry {
    pub source_hash: String,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn load(path: &Path) -> Result<CacheFile> {
    if !path.is_file() {
        return Ok(CacheFile {
            version: CACHE_VERSION,
            files: BTreeMap::new(),
        });
    }
    let source = fs::read_to_string(path)
        .with_context(|| format!("{} 캐시를 읽을 수 없습니다", path.display()))?;
    let cache = serde_json::from_str::<CacheFile>(&source)
        .with_context(|| format!("{} 캐시 형식이 올바르지 않습니다", path.display()))?;
    if cache.version != CACHE_VERSION {
        return Ok(CacheFile {
            version: CACHE_VERSION,
            files: BTreeMap::new(),
        });
    }
    Ok(cache)
}

pub fn save(path: &Path, cache: &CacheFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("{} 캐시 디렉터리를 만들 수 없습니다", parent.display()))?;
    }
    let source = serde_json::to_vec_pretty(cache)?;
    write_atomic_bytes(path, &source)
}

#[must_use]
pub fn source_hash(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}

pub fn write_atomic_text(path: &Path, text: &str) -> Result<()> {
    write_atomic_bytes(path, text.as_bytes())
}

pub fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("{} 디렉터리를 만들 수 없습니다", parent.display()))?;
    let metadata = path.metadata().ok();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let temporary = parent.join(format!(
        ".{}.geullint-{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file"),
        stamp
    ));
    fs::write(&temporary, bytes)
        .with_context(|| format!("{} 임시 파일을 쓸 수 없습니다", temporary.display()))?;
    if let Some(metadata) = metadata.as_ref() {
        fs::set_permissions(&temporary, metadata.permissions())
            .with_context(|| format!("{} 파일 권한을 보존할 수 없습니다", path.display()))?;
    }

    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(_rename_error) if path.exists() => {
            // Windows does not replace an existing file with rename. Keep a same-directory
            // backup so an interrupted replacement can be restored.
            let backup = parent.join(format!(
                ".{}.geullint-{}.bak",
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("file"),
                stamp
            ));
            fs::rename(path, &backup).with_context(|| {
                format!("{} 기존 파일을 교체 준비할 수 없습니다", path.display())
            })?;
            match fs::rename(&temporary, path) {
                Ok(()) => {
                    let _ = fs::remove_file(backup);
                    Ok(())
                }
                Err(error) => {
                    let _ = fs::rename(backup, path);
                    let _ = fs::remove_file(&temporary);
                    Err(error).with_context(|| {
                        format!("{} 파일을 원자적으로 교체할 수 없습니다", path.display())
                    })
                }
            }
        }
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(error)
                .with_context(|| format!("{} 파일을 원자적으로 교체할 수 없습니다", path.display()))
        }
    }
}
