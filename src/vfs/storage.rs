use clap::Parser;

use std::env::current_exe;
use std::fs::{self, File};
use std::io::Error;
use std::io::{self, BufRead, ErrorKind};
use std::path::Path;

// paths
#[derive(Parser, Debug, Clone)]
pub struct VFSArgs {
    #[arg(long, default_value = "./storage")]
    pub storage: Option<String>,

    #[arg(long)]
    pub startapp: Option<String>,
}

#[derive(Clone, Debug)]
pub enum VFSNode {
    File {
        name: String,
    },
    Dir {
        name: String,
        children: Vec<VFSNode>,
    },
}
pub struct VFS {
    root: VFSNode,
    sys_path_name: String,
    current_path: String,
    history: Vec<String>,
}

impl VFSArgs {
    pub fn get_init_commands(&self) -> Vec<String> {
        if let Some(path) = &self.startapp {
            if Path::new(path).exists() {
                if let Ok(file) = File::open(path) {
                    let reader = io::BufReader::new(file);
                    return reader.lines().filter_map(|line| line.ok()).collect();
                }
            }
        }
        vec![]
    }
}

impl VFS {
    pub fn new(storage_path: String) -> Result<Self, Error> {
        let mut root = VFSNode::Dir {
            name: "/".to_string(),
            children: vec![],
        };
        VFS::init_dir_reader(storage_path.clone(), &mut root).unwrap();

        Ok(VFS {
            root: root,
            sys_path_name: storage_path.clone(),
            current_path: "/".to_string(),
            history: vec![],
        })
    }

    fn init_dir_reader(sys_path: String, node: &mut VFSNode) -> std::io::Result<()> {
        match node {
            VFSNode::Dir { name: _, children } => {
                for entry in fs::read_dir(sys_path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    let entry_name = entry.file_name().into_string().unwrap_or_default();

                    let mut child_node = if entry_path.is_dir() {
                        VFSNode::Dir {
                            name: entry_name.clone(),
                            children: vec![],
                        }
                    } else {
                        VFSNode::File {
                            name: entry_name.clone(),
                        }
                    };

                    if entry_path.is_dir() {
                        Self::init_dir_reader(
                            entry_path.to_str().unwrap().to_string(),
                            &mut child_node,
                        )?;
                    }

                    children.push(child_node);
                }
            }

            VFSNode::File { name: _ } => {}
        }

        Ok(())
    }

    pub fn get_path_from_node(&self, node: &VFSNode) -> Result<String, Error> {
        fn dfs(current: &VFSNode, target: &VFSNode, path: &mut Vec<String>) -> bool {
            if std::ptr::eq(current, target) {
                return true;
            }

            match current {
                VFSNode::Dir { children, .. } => {
                    for child in children {
                        match child {
                            VFSNode::Dir { name, .. } | VFSNode::File { name } => {
                                path.push(name.clone());
                            }
                        }

                        if dfs(child, target, path) {
                            return true;
                        }

                        path.pop();
                    }
                }
                _ => {}
            }

            false
        }

        let mut path = Vec::new();

        if std::ptr::eq(&self.root, node) {
            return Ok("/".to_string());
        }

        if dfs(&self.root, node, &mut path) {
            Ok(format!("/{}", path.join("/")))
        } else {
            Err(Error::new(ErrorKind::NotFound, "node not found"))
        }
    }

    fn get_node_from_path(&mut self, path: &String) -> Result<&VFSNode, Error> {
        let mut current_obj: &VFSNode = &self.root;

        let full_path = if path.starts_with("/") {
            path.clone()
        } else if self.current_path == "/" {
            format!("/{}", path)
        } else {
            format!("{}/{}", self.current_path, path)
        };

        let parts: Vec<&str> = full_path.split("/").filter(|s| !s.is_empty()).collect();

        if parts.len() == 0 {
            return Ok(&self.root);
        }

        for obj in parts {
            match current_obj {
                VFSNode::Dir { children, .. } => {
                    println!("{:?}", current_obj);
                    if let Some(child) = children.iter().find(|c| match c {
                        VFSNode::Dir { name, .. } => name == obj,
                        VFSNode::File { name } => name == obj,
                    }) {
                        current_obj = child;
                    } else {
                        return Err(Error::new(
                            ErrorKind::NotFound,
                            format!("dir not found: {}", obj),
                        ));
                    }
                }
                VFSNode::File { .. } => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("{} is a file, not a directory", obj),
                    ));
                }
            }
        }

        Ok(current_obj)
    }

    pub fn change_dir(&mut self, args: Vec<String>) -> Result<&VFSNode, Error> {
        /*
            TODO: FIX THIS!
            It's ignoring borrow checker, bad practice
        */
        let path = if args.is_empty() {
            "/".to_string()
        } else if args.len() > 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "too many args"));
        } else {
            args[0].clone()
        };

        let node_ptr: *const VFSNode = {
            let node_ref = self.get_node_from_path(&path)?;
            node_ref as *const VFSNode
        };

        let new_path = {
            let node_ref = unsafe { &*node_ptr };
            self.get_path_from_node(node_ref)?
        };

        self.current_path = new_path;

        Ok(unsafe { &*node_ptr })
    }

    pub fn list_dir(&mut self, args: Vec<String>) -> Result<&Vec<VFSNode>, Error> {
        let path = if args.is_empty() {
            self.current_path.clone()
        } else if args.len() > 1 {
            return Err(Error::new(ErrorKind::InvalidInput, "too many args"));
        } else {
            args[0].clone()
        };

        let node = self.get_node_from_path(&path)?;
        match node {
            VFSNode::Dir { name: _, children } => {
                Ok(children)
            }
            VFSNode::File { name } => {
                return Err(Error::new(ErrorKind::InvalidInput, format!("{}: not a dir", name)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let vfs = VFS::new("./storage".to_string()).unwrap();
        println!("{:?} {}", vfs.root, vfs.current_path);
    }

    #[test]
    fn test_found_dir() {
        let mut vfs = VFS::new("./storage".to_string()).unwrap();
        vfs.get_node_from_path(&"/".to_string());
        vfs.get_node_from_path(&"xd/double/r".to_string());
        vfs.get_node_from_path(&"/xddddd".to_string());
        vfs.get_node_from_path(&"test/second_dir".to_string());
    }
}
