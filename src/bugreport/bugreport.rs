use glob::glob;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use super::bugreport_txt::BugreportTxt;

struct BugreportDirs {
    bugreport_txt: PathBuf,
    anr_files_dir: PathBuf,
    miuilog_reboot_dir: PathBuf,
    miuilog_scout_dir: PathBuf,
}

impl BugreportDirs {
    fn new() -> Self {
        BugreportDirs {
            bugreport_txt: PathBuf::new(),
            anr_files_dir: PathBuf::new(),
            miuilog_reboot_dir: PathBuf::new(),
            miuilog_scout_dir: PathBuf::new(),
        }
    }
}

pub struct Bugreport {
    bugreport_txt: BugreportTxt,
    anr_files: Vec<String>,
    miuilog_reboots: Vec<String>,
    miuilog_scouts: Vec<String>,
}

impl Bugreport {
    pub fn new(bugreport_zip_path: &Path) -> Self {
        let bugreport_txt = BugreportTxt::new(bugreport_zip_path).unwrap();
        Bugreport {
            bugreport_txt,
            anr_files: Vec::new(),
            miuilog_reboots: Vec::new(),
            miuilog_scouts: Vec::new(),
        }
    }

    fn unzip(path: &Path) -> io::Result<()> {
        // Unzip the bug report file
        let zip_file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(zip_file)?;
        let base_dir = path
            .file_stem()
            .unwrap_or_else(|| std::ffi::OsStr::new("."));
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out_path = Path::new(base_dir).join(file.name());
            if file.is_dir() {
                std::fs::create_dir_all(&out_path)?;
            } else {
                let mut outfile = std::fs::File::create(&out_path)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok(())
    }

    pub fn load(&mut self) -> io::Result<()> {
        // Load the bug report and extract relevant information
        self.bugreport_txt.load()?;
        // self.anr_files = self.bugreport_txt.get_anr_files();
        // self.miuilog_reboots = self.bugreport_txt.get_miuilog_reboots();
        // self.miuilog_scouts = self.bugreport_txt.get_miuilog_scouts();
        Ok(())
    }
}

fn extract(
    feedback_id: u64,
    bugreport_zip: &Path,
    user_feedback_path: &Path,
) -> Result<BugreportDirs, Box<dyn std::error::Error>> {
    // Create feedback directory
    let feedback_dir = user_feedback_path.join(feedback_id.to_string());
    fs::create_dir_all(&feedback_dir)?;

    // Extract and remove the initial bugreport zip
    if let Err(e) = unzip_and_delete(bugreport_zip, &feedback_dir) {
        eprintln!("Failed to extract initial bugreport: {}", e);
    }

    // Find subsequent bugreport zip
    let pattern = feedback_dir.join("bugreport*.zip");
    let matches = glob(pattern.to_str().unwrap())?;
    let bugreport_zip_path = matches.filter_map(Result::ok).next();

    let bugreport_zip_path = match bugreport_zip_path {
        Some(path) => path,
        None => {
            eprintln!("No bugreport*.zip found");
            // TODO: modify this to actual paths
            return Ok(BugreportDirs {
                bugreport_txt: feedback_dir.join("bugreport.txt"),
                anr_files_dir: feedback_dir.join("anr_files"),
                miuilog_reboot_dir: feedback_dir.join("miuilog_reboot"),
                miuilog_scout_dir: feedback_dir.join("miuilog_scout"),
            });
        }
    };

    // Create extraction directory
    let bugreport_dir = feedback_dir.join(
        bugreport_zip_path
            .file_stem()
            .ok_or("Invalid zip filename")?
            .to_str()
            .ok_or("Non-UTF8 filename")?,
    );
    fs::create_dir_all(&bugreport_dir)?;

    // Extract and remove secondary zip
    if let Err(e) = unzip_and_delete(&bugreport_zip_path, &bugreport_dir) {
        eprintln!("Failed to extract secondary bugreport: {}", e);
    }

    // Check for reboot directory
    let reboot_mqs_dir = bugreport_dir
        .join("FS")
        .join("data")
        .join("miuilog")
        .join("stability")
        .join("reboot");

    if reboot_mqs_dir.is_dir() {
        let zip_pattern = reboot_mqs_dir.join("*.zip");
        for zip_path in glob(zip_pattern.to_str().unwrap())?.filter_map(Result::ok) {
            let extract_dir = zip_path.with_extension("");
            fs::create_dir_all(&extract_dir)?;
            if let Err(e) = unzip_and_delete(&zip_path, &extract_dir) {
                eprintln!("Failed to extract nested zip: {}", e);
            }
        }
    } else {
        eprintln!("No reboot directory found");
    }

    // TODO: modify this to actual paths
    Ok(BugreportDirs {
        bugreport_txt: feedback_dir.join("bugreport.txt"),
        anr_files_dir: feedback_dir.join("anr_files"),
        miuilog_reboot_dir: feedback_dir.join("miuilog_reboot"),
        miuilog_scout_dir: feedback_dir.join("miuilog_scout"),
    })
}

fn unzip_and_delete(zip_path: &Path, dest_dir: &Path) -> io::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(dest_dir)?;
    fs::remove_file(zip_path)?;
    Ok(())
}
