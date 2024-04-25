use clap::Args;

#[derive(Args)]
pub struct AffectedArgs {
    /// Comma separated list of projects to check
    #[arg(short, long, default_value = "all")]
    pub projects: String
}

pub fn list_affected(args: &AffectedArgs) {
    let projects = args.projects.split(",").collect::<Vec<&str>>();
    println!("Listing affected projects: {:?}", projects);
}