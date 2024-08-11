use std::{fs, io};
use std::path::PathBuf;

use anyhow::Result;


pub fn extract(from: &str, to: &str) -> Result<()>{
    
    let from_file = fs::File::open(from)?;

    let mut archive = zip::ZipArchive::new(from_file)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        let out_path = PathBuf::from(to).join(out_path);
        debug!("{:?}", out_path);
        if file.is_dir() {
            fs::create_dir_all(&out_path)?
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
            }
        }
    }
    
    Ok(())
}