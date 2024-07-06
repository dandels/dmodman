#[allow(non_camel_case_types)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

const BLOCK_SIZE: usize = 10240;

struct ArchiveWrapper {
    archive: *mut bindings::archive,
    offset: i64,
}

unsafe impl Send for ArchiveWrapper {}
unsafe impl Sync for ArchiveWrapper {}

#[derive(Clone)]
pub struct Archive {
    inner: Arc<Mutex<ArchiveWrapper>>,
}

impl Archive {
    pub async fn open(filename: String) -> Result<Self, ArchiveError> {
        let wrapper = task::spawn_blocking(move || ArchiveWrapper::open(&filename)).await.unwrap()?;
        Ok(Self {
            inner: Arc::new(Mutex::new(wrapper)),
        })
    }

    pub async fn next(&self) -> Option<Result<Entry, ArchiveError>> {
        let mut lock = self.inner.clone().lock_owned().await;
        task::spawn_blocking(move || lock.next()).await.unwrap()
    }

    pub async fn read_data_block(&self) -> (i32, Option<Vec<u8>>) {
        let mut lock = self.clone().inner.lock_owned().await;
        task::spawn_blocking(move || lock.read_data_block()).await.unwrap()
    }

    pub async fn get_err_msg(&self) -> String {
        self.inner.lock().await.get_err_msg()
    }
}

impl ArchiveWrapper {
    pub fn open(filename: &str) -> Result<Self, ArchiveError> {
        let archive = unsafe {
            let archive = bindings::archive_read_new();
            bindings::archive_read_support_format_all(archive);
            let c_filename = CString::new(filename).unwrap();
            let status = bindings::archive_read_open_filename(archive, c_filename.as_ptr(), BLOCK_SIZE);
            match status {
                0 => Ok(archive),
                code => {
                    let err_msg = CStr::from_ptr(bindings::archive_error_string(archive)).to_string_lossy().to_string();
                    Err(ArchiveError::from_err_code(code, err_msg))
                }
            }
        }?;

        Ok(Self { archive, offset: 0 })
    }

    pub fn next(&mut self) -> Option<Result<Entry, ArchiveError>> {
        unsafe {
            let mut entry: *mut bindings::archive_entry = ptr::null_mut();
            crate::logger::log_to_file("Trying to get next entry...");
            match bindings::archive_read_next_header(self.archive, &mut entry) {
                0 => Some(Ok(Entry::new(entry))),
                bindings::ARCHIVE_EOF => None,
                code => {
                    let err_msg =
                        CStr::from_ptr(bindings::archive_error_string(self.archive)).to_string_lossy().to_string();
                    Some(Err(ArchiveError::from_err_code(code, err_msg)))
                }
            }
        }
    }

    pub fn read_data_block(&mut self) -> (i32, Option<Vec<u8>>) {
        let mut buf = std::ptr::null();
        let mut size: libc::size_t = 0;
        let status = unsafe { bindings::archive_read_data_block(self.archive, &mut buf, &mut size, &mut self.offset) };
        if let bindings::ARCHIVE_OK | bindings::ARCHIVE_WARN = status {
            // When reading RAR files, libarchive returns OK on every second read with no data...
            if buf.is_null() {
                (status, None)
            } else {
                // I hope calling .to_vec() is inexpensive since we don't want to copy unextracted data around in memory
                (status, Some(unsafe { std::slice::from_raw_parts(buf as *const u8, size).to_vec() }))
            }
        } else {
            (status, None)
        }
    }

    pub fn get_err_msg(&self) -> String {
        unsafe { CStr::from_ptr(bindings::archive_error_string(self.archive)).to_string_lossy().to_string() }
    }
}

impl Drop for ArchiveWrapper {
    fn drop(&mut self) {
        unsafe {
            bindings::archive_read_free(self.archive);
        }
    }
}

struct EntryWrapper {
    entry: *mut bindings::archive_entry,
}
pub struct Entry {
    inner: Arc<Mutex<EntryWrapper>>,
}

unsafe impl Send for EntryWrapper {}
unsafe impl Sync for EntryWrapper {}

impl Entry {
    pub fn new(entry: *mut bindings::archive_entry) -> Self {
        let wrapper = EntryWrapper { entry };
        Self {
            inner: Arc::new(Mutex::new(wrapper)),
        }
    }

    pub async fn is_dir(&self) -> bool {
        self.inner.lock().await.is_dir()
    }

    pub async fn path(&self) -> PathBuf {
        self.inner.lock().await.path()
    }
}

impl EntryWrapper {
    // TODO allow archives to include symlinks etc if they are sanitized
    fn is_dir(&self) -> bool {
        match unsafe { bindings::archive_entry_filetype(self.entry) } {
            libc::S_IFDIR => true,
            libc::S_IFREG => false,
            stat => {
                println!("File was neither file nor directory, stat is {stat}.");
                false
            }
        }
    }

    fn path(&self) -> PathBuf {
        unsafe {
            // Does this crash with sigsegv address boundary error?
            let ptr = bindings::archive_entry_pathname(self.entry);
            let slice = CStr::from_ptr(ptr);
            let osstr = OsStr::from_bytes(slice.to_bytes());
            osstr.into()
        }
    }
}

#[derive(Debug)]
pub enum ArchiveError {
    Failed(String),
    Fatal(String),
    Unhandled(String),
    Warn(String),
}

impl ArchiveError {
    pub fn from_err_code(code: i32, err_msg: String) -> Self {
        match code {
            bindings::ARCHIVE_WARN => ArchiveError::Warn(err_msg),
            bindings::ARCHIVE_FAILED => ArchiveError::Failed(err_msg),
            bindings::ARCHIVE_FATAL => ArchiveError::Fatal(err_msg),
            _ => ArchiveError::Unhandled(err_msg),
        }
    }
}

impl std::error::Error for ArchiveError {}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed(msg) => f.write_str(msg),
            Self::Fatal(msg) => f.write_str(msg),
            Self::Unhandled(msg) => f.write_str(msg),
            Self::Warn(msg) => f.write_str(msg),
        }
    }
}
