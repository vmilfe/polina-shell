use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// paths
#[derive(Parser, Debug, Clone)]
pub struct VFSArgs {
    #[arg(long, default_value = "./storage")]
    pub storage: Option<String>,

    #[arg(long)]
    pub startapp: Option<String>,
}


impl VFSArgs {
    pub fn get_init_commands(&self) -> Vec<String> {
        if let Some(path) = &self.startapp {
            if Path::new(path).exists() {
                if let Ok(file) = File::open(path) {
                    let reader = io::BufReader::new(file);
                    return reader
                        .lines()
                        .filter_map(|line| line.ok()) 
                        .collect();
                }
            }
        }
        vec![]
    }
}