use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct RamFile {
    pub name: String,
    pub data: Vec<u8>,
    pub size: usize,
    pub created_at: u64,
    pub file_type: FileType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Text,
    Binary,
    Executable,
    Directory,
    Unknown,
}

impl FileType {
    pub fn to_string(&self) -> &'static str {
        match self {
            FileType::Text => "TXT",
            FileType::Binary => "BIN",
            FileType::Executable => "EXE",
            FileType::Directory => "DIR",
            FileType::Unknown => "???",
        }
    }
}

pub struct RamFs {
    pub files: Vec<RamFile>,
}

impl RamFs {
    pub fn new() -> Self {
        RamFs { files: Vec::new() }
    }
    
    pub fn create_file(&mut self, name: &str, data: &[u8]) {
        let file_type = Self::detect_file_type(name, data);
        self.files.push(RamFile {
            name: name.into(),
            data: data.to_vec(),
            size: data.len(),
            created_at: Self::get_timestamp(),
            file_type,
        });
    }
    
    pub fn read_file(&self, name: &str) -> Option<&[u8]> {
        self.files.iter().find(|f| f.name == name).map(|f| f.data.as_slice())
    }
    
    pub fn write_file(&mut self, name: &str, data: &[u8]) -> bool {
        if let Some(file) = self.files.iter_mut().find(|f| f.name == name) {
            file.data = data.to_vec();
            file.size = data.len();
            file.file_type = Self::detect_file_type(name, data);
            true
        } else {
            false
        }
    }
    
    pub fn list_files(&self) -> Vec<&str> {
        self.files.iter().map(|f| f.name.as_str()).collect()
    }
    
    pub fn list_files_detailed(&self) -> Vec<&RamFile> {
        self.files.iter().collect()
    }
    
    pub fn get_file_info(&self, name: &str) -> Option<&RamFile> {
        self.files.iter().find(|f| f.name == name)
    }
    
    fn detect_file_type(name: &str, data: &[u8]) -> FileType {
        if name.ends_with(".txt") || name.ends_with(".md") || name.ends_with(".log") {
            return FileType::Text;
        }
        if name.ends_with(".exe") || name.ends_with(".bin") || name.ends_with(".elf") {
            return FileType::Executable;
        }
        if name.ends_with("/") || name.ends_with("\\") {
            return FileType::Directory;
        }
        
        if data.iter().all(|&b| b >= 32 && b <= 126 || b == b'\n' || b == b'\r' || b == b'\t') {
            FileType::Text
        } else {
            FileType::Binary
        }
    }
    
    fn get_timestamp() -> u64 {
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            COUNTER
        }
    }
}

pub static mut RAMFS: Option<RamFs> = None;

pub fn init_ramfs() -> bool {
    unsafe {
        RAMFS = Some(RamFs::new());
        if RAMFS.is_some() {
            crate::print_ok!("RAMFS initialized");
            true
        } else {
            crate::print_fail!("RAMFS initialization failed");
            false
        }
    }
}

pub fn ramfs_mut() -> &'static mut RamFs {
    unsafe { RAMFS.as_mut().unwrap() }
}