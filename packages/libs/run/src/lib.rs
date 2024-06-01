use clap::Args;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
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
#[derive(Clone)]
pub struct ProjectStats {
    pub project: String,
    pub command: String,
    pub result: String,
    pub fetch_duration: std::time::Duration,
    pub run_duration: std::time::Duration,
    pub cache_duration: std::time::Duration,
}

/// An utility function that takes another function and calculates the time it takes to run it
/// While also returning the result of the function
fn time_it<F, T>(f: F) -> (T, std::time::Duration)
where
    F: FnOnce() -> T,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Print the results of the run
/// The results are printed in the following format:
/// Project: <project_name> -> Command: <command> -> Result: <result> -> Run Duration: <run_duration> -> Cache Duration: <cache_duration>
/// The results are printed in the order they were processed, and seperated by cache hits and cache misses
fn print_results(stats: Vec<ProjectStats>) {
    // seperate the results in terms of cache hits and cache misses
    let cache_misses: Vec<ProjectStats> = stats.iter()
        .filter(|s| s.fetch_duration.as_secs() == 0)
        .cloned()
        .collect();
    println!("Projects with cache misses:");
    cache_misses.iter()
        .for_each(|s| {
            println!("{} -> Fetch Duration: {:?} -> Run Duration: {:?} -> Cache Duration: {:?}", s.project, s.fetch_duration, s.run_duration, s.cache_duration);
        });
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
    println!("Running command: {} in project: {} path: {}", command, project.name, project.path);
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&project.path)
        .output();
    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            if output.status.success() {
                Ok(output_str)
            } else {
                if output.status.code().is_some() && output.status.code().unwrap() == 254 {
                    return Ok("".to_string());
                }
                // if stdout has Command failed with ENOENT, it means the command was not found, skip it
                if output_str.contains("Command failed with ENOENT") {
                    // Command was not found FIXME: handle this better
                    println!("WARNING!: Command not found for project: {} -> Command: {}", project.name, command);
                    return Ok("".to_string());
                }
                panic!("Error running command: {}\n Command failed with the message below:\n{:?}", command, output);
            }
        }
        Err(e) => {
            panic!("Error running command: {}\n Command failed with the message below:\n{:?}", command, e.to_string());
        }
    }
}

fn try_fetch_cache(project: &init::Project, command: &str, hash: &str, cache: &cacher::Cache, root_path: String) -> Result<(String, Duration, Duration, Duration), String> {
    let (result, get_duration) = time_it(|| cache.get(project, command, hash, root_path));
    match result {
        Some(result) => Ok((result, get_duration, Duration::from_secs(0), Duration::from_secs(0))),
        None => {
            let (result, run_duration) = time_it(|| run_command(project, command));
            match result {
                Ok(result) => {
                    let (_, cache_duration ) = time_it(|| cache.put(project, command, hash, &result));
                    println!("Project: {} -> Command: {} -> Result: {} -> Run Duration: {:?} -> Cache Duration: {:?}", project.name, command, result, run_duration, cache_duration);
                    Ok((result, get_duration, run_duration, cache_duration))
                }
                Err(e) => Err(e)
            }
        }
    }
}

pub fn run(args: &RunArgs, workspace: Vec<init::Project>, root_path: String) {
    let current_workspace = Arc::new(Mutex::new(workspace.clone()));
    let global_hashmap: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let cache: Arc<cacher::Cache> = Arc::new(cacher::Cache::new(&args.cache_type));
    let stats: Arc<Mutex<Vec<ProjectStats>>> = Arc::new(Mutex::new(vec![]));
    loop {
        let projects = get_dependency_free_projects(current_workspace.lock().unwrap().clone());
        if projects.len() == 0 {
            break;
        }
        let mut handles: Vec<thread::JoinHandle<()>> = vec![];
        for project in projects {

            let current_workspace = Arc::clone(&current_workspace);
            let global_hashmap = Arc::clone(&global_hashmap);
            let stats = Arc::clone(&stats);
            let cache = Arc::clone(&cache);
            let command = args.command.clone();
            let root_path = root_path.clone();

            let handle = thread::spawn(move || {
                let mut hashmap = global_hashmap.lock().unwrap();
                let hash = hasher::hash(&project, &command, hashmap.clone());
                match try_fetch_cache(&project, &command, &hash, &cache, root_path) {
                    Ok(result) => {
                        let mut stats = stats.lock().unwrap();
                        stats.push(ProjectStats {
                            project: project.name.clone(),
                            command: command.clone(),
                            result: result.0,
                            fetch_duration: result.1,
                            run_duration: result.2,
                            cache_duration: result.3,
                        });
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

    print_results(stats.lock().unwrap().clone());
}