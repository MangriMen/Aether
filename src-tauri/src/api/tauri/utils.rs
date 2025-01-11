use std::path::PathBuf;

#[tauri::command]
pub fn reveal_in_explorer(path: String, exact: bool) -> Result<(), String> {
    let path = PathBuf::from(path);
    crate::utils::file::reveal_in_explorer(&path, exact).map_err(|err| err.to_string())
}
