// this class will create and manage folders.

struct FolderManager {
    base_path: String,
}

impl FolderManager {
    pub fn new(base_path: &str) -> Self {
        FolderManager {
            base_path: base_path.to_string(),
        }
    }

    pub fn create_folder(&self, folder_name: &str) -> std::io::Result<()> {
        let path = std::path::Path::new(&self.base_path).join(folder_name);
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn delete_folder(&self, folder_name: &str) -> std::io::Result<()> {
        let path = std::path::Path::new(&self.base_path).join(folder_name);
        if path.exists() {
            std::fs::remove_dir_all(path)?;
        }
        Ok(())
    }
    pub fn list_folders(&self) -> std::io::Result<Vec<String>> {
        let path = std::path::Path::new(&self.base_path);
        if path.exists() && path.is_dir() {
            let mut folders = Vec::new();
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        folders.push(name.to_string());
                    }
                }
            }
            Ok(folders)
        } else {
            Ok(Vec::new())
        }
    }
}