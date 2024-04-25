use sha2::{Digest, Sha256};
use base64ct::{Base64, Encoding};
use std::{collections::HashMap, fs};

const IGNORED_FILES: [&str; 2] = [".git", "node_modules"];

pub fn hash(project: &init::Project, command: &str, global_hashmap: HashMap<String,String>) -> String{    
    let mut hasher = Sha256::new();
    hasher.update(project.name.as_bytes());
    project.dependencies.iter().for_each(|(name, _)| {
        if let Some(hash) = global_hashmap.get(name) {
            hasher.update(hash.as_bytes());
        }
    });
    hasher.update(command.as_bytes());

    fn hash_dir (path: &str, hasher: &mut Sha256) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if IGNORED_FILES.contains(&entry.file_name().to_str().unwrap()) {
                        continue;
                    }
                    if metadata.is_file() {
                        if let Ok(file_content) = fs::read(entry.path()) {
                            hasher.update(&file_content);
                        }
                    }
                    if metadata.is_dir() {
                        hash_dir(entry.path().to_str().unwrap(), hasher);
                    }
                }
            }
        }
    }

    hash_dir(&project.path, &mut hasher);
    let result = hasher.finalize();
    Base64::encode_string(&result)
}