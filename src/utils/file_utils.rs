use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

// 获取日志文件
pub fn get_log_file(file_path: &str) -> Option<PathBuf> {
    if is_empty(file_path) {
        return None;
    }

    match get_file_type(file_path) {
        0 => get_logfile_from_directory(Path::new(file_path)),
        1 => {
            if let Some(unzip_dir) = up_zip_file(file_path) {
                get_logfile_from_directory(&unzip_dir)
            } else {
                None
            }
        }
        2 => None, // RAR 文件暂不支持
        _ => Some(PathBuf::from(file_path)),
    }
}

// 获取文件类型
pub fn get_file_type(file_path: &str) -> i32 {
    if is_empty(file_path) {
        return -1;
    }

    let path = Path::new(file_path);
    if path.is_dir() {
        return 0;
    }

    if let Some(suffix) = get_suffix(file_path) {
        match suffix.to_lowercase().as_str() {
            "zip" | "gz" => 1,
            "rar" => 2,
            "log" | "txt" | "bugreport" => 3,
            _ => -1,
        }
    } else {
        -1
    }
}

// 获取文件后缀
fn get_suffix(file_path: &str) -> Option<String> {
    if is_empty(file_path) {
        return None;
    }

    Path::new(file_path)
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase())
}

// 从目录中获取日志文件
fn get_logfile_from_directory(folder: &Path) -> Option<PathBuf> {
    if !folder.is_dir() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if get_file_type(path.to_str().unwrap()) == 3 {
                return Some(path);
            }
        }
    }
    None
}

// 解压 ZIP 文件
fn up_zip_file(zip_file_path: &str) -> Option<PathBuf> {
    let zip_file = Path::new(zip_file_path);
    let dest_dir = zip_file.parent().unwrap().join("upZipLogFolder");

    if dest_dir.exists() {
        fs::remove_dir_all(&dest_dir).ok()?;
    }
    fs::create_dir_all(&dest_dir).ok()?;

    if let Ok(file) = File::open(zip_file) {
        if let Ok(mut archive) = ZipArchive::new(file) {
            for i in 0..archive.len() {
                if let Ok(mut file) = archive.by_index(i) {
                    let out_path = dest_dir.join(file.name());
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent).ok()?;
                    }
                    if let Ok(mut out_file) = File::create(&out_path) {
                        io::copy(&mut file, &mut out_file).ok()?;
                    }
                }
            }
            return Some(dest_dir);
        }
    }
    None
}

// 写入一行到文件
pub fn write_line_to_file(line: &str, writer: &mut BufWriter<File>) -> io::Result<()> {
    if !is_empty(line) {
        writeln!(writer, "{}", line)?;
        writer.flush()?;
    }
    Ok(())
}

// 检查字符串是否为空
pub fn is_empty(s: &str) -> bool {
    s.trim().is_empty()
}

// 获取输出目录
pub fn get_output_dir(dir: &str) -> PathBuf {
    let out_dir = Path::new(dir).join("out");
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).unwrap();
    }
    out_dir
}

// 检查文件是否存在
pub fn is_exists(file: &Path) -> bool {
    file.exists()
}
