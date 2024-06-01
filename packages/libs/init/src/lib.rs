use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use glob::glob;

#[derive(Debug)]
pub struct Project {
    pub path: String,
    pub name: String,
    pub dependencies: HashMap<String, bool>,
}

impl Clone for Project {
    fn clone(&self) -> Self {
        Project {
            path: self.path.clone(),
            name: self.name.clone(),
            dependencies: self.dependencies.clone(),
        }
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {:?}", self.name, self.dependencies)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Package {
    name: String,
    dependencies: Option<std::collections::HashMap<String, String>>,
    dev_dependencies: Option<std::collections::HashMap<String, String>>,
    workspaces: Option<Vec<String>>,
}

fn read_package_json<P: AsRef<Path>>(path: P) -> Result<Package, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let pjson = serde_json::from_reader(reader)?;
    Ok(pjson)
}

pub fn get_workspace(path: &str, current_workspace: &mut Vec<Project>) -> Result<(), Box<dyn Error>> {
    let package_json_path = format!("{}/package.json", path);
    let root_pjson = read_package_json(package_json_path).unwrap();
    if let Some(workspaces) = root_pjson.workspaces {
        for workspace in workspaces {
            let workspace_path = format!("{}/{}", path, workspace);
            for entry in glob(&workspace_path).unwrap() {
                let entry = entry.unwrap();
                let package_json_path = format!("{}/package.json", entry.to_str().unwrap());
                let pjson = read_package_json(package_json_path.clone()).unwrap();
                let project_name = pjson.name.clone();
                current_workspace.push(Project {
                    path: entry.to_str().unwrap().to_string(),
                    name: pjson.name,
                    dependencies: get_dependencies(&package_json_path, &project_name).unwrap(),
                });
            }
        }
    }
    Ok(())
}

pub fn get_dependencies(package_json_path: &str, project_name: &str) -> Result<HashMap<String, bool>, Box<dyn Error>> {
    let pjson = read_package_json(package_json_path).unwrap();
    let mut dependencies = HashMap::new();
    if let Some(deps) = pjson.dependencies {
        deps.iter().filter(|k: &(&String, &String)| k.1 == "workspace:*").for_each(|k: (&String, &String)| {
            if k.0 != project_name {
                dependencies.insert(k.0.clone(), false);
            }
        });
    }
    if let Some(deps) = pjson.dev_dependencies {
        deps.iter().filter(|k: &(&String, &String)| k.1 == "workspace:*").for_each(|k: (&String, &String)| {
            if k.0 != project_name {
                dependencies.insert(k.0.clone(), false);
            }
        });
    }
    Ok(dependencies)
}