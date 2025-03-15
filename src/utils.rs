use std::fs;

pub fn clean_directory(folder_path:&str) -> Result<String, String> {
    //! Delete all the CSV files in the folder in input.
    let entries = fs::read_dir(folder_path)
        .map_err(|error| format!("{}",error))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("{}",error))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "csv" {
                    fs::remove_file(&path).map_err(|error|format!("{}",error))?;
                }
            }
        }
    }
    Ok("Successfully clean the directory !".to_string())
}