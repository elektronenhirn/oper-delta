use serde_xml_rs::from_reader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use anyhow::{anyhow,Result};
use serde::Deserialize;

pub fn parse(path: &Path) -> Result<Manifest>{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut manifest: Manifest = from_reader(reader)?;
    let includes : Vec<String> = manifest.includes.iter().map(|i| i.name.clone()).collect();
    for include in &includes {
        let path = path.with_file_name(include);
        let child = parse(&path).or_else(|e| Err(anyhow!("Failed to parse {}: {}", include, e)))?;
        manifest.append(&child);
    }
    Ok(manifest)
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(rename = "project", default)]
    pub projects: Vec<Project>,
    #[serde(rename = "include", default)]
    pub includes: Vec<Include>,
}

impl Manifest {
    pub fn append(&mut self, manifest: &Manifest){
        let projects = &manifest.projects;
        self.projects.extend(projects.iter().cloned());
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub groups: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Include {
    pub name: String,
}

#[test]
fn test_parse() {
    let manifest = parse(Path::new("test/upstream.xml")).unwrap();
    assert_eq!(manifest.projects.len(), 2);
}

#[test]
fn test_parse_recursive() {
    let manifest = parse(Path::new("test/default.xml")).unwrap();
    assert_eq!(manifest.projects.len(), 3);
}