// a cacher that can get and put results from a CLI command
// the cacher can be a remote service or a local cache
// to get the cache, the cacher needs to know the project, the command and the hash
// to put the cache, the cacher needs to know the project, the command, the hash and the result
use init;

#[derive(Debug)]
pub enum CacheType {
    Remote,
    Local,
}

pub struct Cache {
    cache_type: CacheType,
}

impl Cache {
    pub fn new(c_type: &String) -> Cache {
        let c_type = match c_type.as_str() {
            "remote" => CacheType::Remote,
            _ => CacheType::Local,
        };
        Cache {
            cache_type: c_type,
        }
    }
    pub fn get(&self, project: &init::Project, command: &str, hash: &str, root: String) -> Option<String> {
        let command = command.replace(" ", "-");
        match self.cache_type {
            CacheType::Remote => {
                println!("Fetching remote cache: {}/{}/{}", project.name, command, hash);
                None
            }
            CacheType::Local => {
                // cache path is root of the project + .renith_cache
                println!("Fetching local cache from {}/.renith_cache", root );
                None
            }
        }
    }

    pub fn put(&self, project: &init::Project, command: &str, hash: &str, result: &str) {
        let command = command.replace(" ", "-");
        match self.cache_type {
            CacheType::Remote => {
                println!("Caching result: {} with key: {}/{}/{}", result, project.name, command, hash);
            }
            CacheType::Local => {
                println!("Caching result: {} with key: {}/{}/{}", result, project.name, command, hash);
            }
        }
    }
}
