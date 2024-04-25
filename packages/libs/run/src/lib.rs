use clap::Args;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use cacher;

#[derive(Args)]
pub struct RunArgs {
    /// Command to check in cache and run if not found.
    /// The result of the command is cached.
    /// If the command fails, the cache is not updated
    #[arg(short, long)]
    pub command: String,
    /// Cache type to use
    /// Can be remote or local
    /// Default is local
    #[arg(long, default_value = "local")]
    pub cache_type: String
}

fn get_dependency_free_projects(workspace: Vec<init::Project>) -> Vec<init::Project>  {
    // Get projects that have all dependencies processed or no dependencies
    workspace.iter()
        .filter(|p| {
            p.dependencies.iter()
                .all(|(_, processed)| *processed)
        })
        .cloned()
        .collect()
}

fn remove_project_from_workspace(workspace: &mut Vec<init::Project>, project: &init::Project) {
    // Mark project as processed in dependencies of other projects
    // Remove project from workspace
    workspace.iter_mut()
        .for_each(|p| {
            p.dependencies.iter_mut()
                .for_each(|(name, processed)| {
                    if name == &project.name {
                        *processed = true;
                    }
                });
        });
    workspace.retain(|p| p.name != project.name);

}

fn run_command(project: &init::Project, command: &str) -> Result<String, String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&project.path)
        .output();
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                panic!("Error running command: {}\n Command failed with the message below:\n{}", command, String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        Err(e) => {
            panic!("Error running command: {}\n Command failed with the message below:\n{}", command, e.to_string());
        }
    }
}

fn try_fetch_cache(project: &init::Project, command: &str, hash: &str, cache: &cacher::Cache) -> Result<String, String> {
    let result = cache.get(project, command, hash);
    match result {
        Some(result) => Ok(result),
        None => {
            let result = run_command(project, command);
            match result {
                Ok(result) => {
                    cache.put(project, command, hash, &result);
                    Ok(result)
                }
                Err(e) => Err(e)
            }
        }
    }
}

pub fn run(args: &RunArgs, workspace: Vec<init::Project>) {
    let current_workspace = Arc::new(Mutex::new(workspace.clone()));
    let global_hashmap: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let cache: Arc<cacher::Cache> = Arc::new(cacher::Cache::new(&args.cache_type));
    loop {
        let projects = get_dependency_free_projects(current_workspace.lock().unwrap().clone());
        if projects.len() == 0 {
            break;
        }
        let mut handles = vec![];
        for project in projects {
            let current_workspace = Arc::clone(&current_workspace);
            let global_hashmap = Arc::clone(&global_hashmap);
            let cache = Arc::clone(&cache);
            let command = args.command.clone();

            let handle = thread::spawn(move || {
                let mut hashmap = global_hashmap.lock().unwrap();
                let hash = hasher::hash(&project, &command, hashmap.clone());
                match try_fetch_cache(&project, &command, &hash, &cache) {
                    Ok(result) => {
                        println!("Project: {} -> Command: {} -> Result: {}", project.name, command, result);
                    }
                    Err(e) => {
                        println!("Project: {} -> Command: {} -> Error: {}", project.name, command, e);
                    }
                }
                
                hashmap.insert(project.name.clone(), hash);
                remove_project_from_workspace(&mut current_workspace.lock().unwrap(), &project);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}