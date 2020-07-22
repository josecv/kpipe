extern crate yaml_rust;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::{self, Read};
use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlLoader};

fn main() {
    let docs = slurp(&mut io::stdin()).unwrap();
    let by_file = split_to_files(&docs);
    for (key, value) in by_file {
        println!("{} / {:?}", key, value);
    }
}

fn split_to_files<'a>(docs: &'a Vec<Yaml>) -> HashMap<&'a String, &'a Yaml> {
    docs.iter()
        .map(|y| match y {
            Yaml::Hash(hash) => {
                hash.get(&Yaml::String("kind".to_string()))
                    .and_then(|val| match val {
                        Yaml::String(x) => Some((x, y)),
                        _ => None,
                    })
            }
            _ => None,
        })
        .filter(|o| o.is_some())
        .map(|o| o.expect("splitting yamls failed unexpectedly"))
        .collect()
}

fn get_yaml_str<'a>(hash: &'a Hash, key: &str) -> Option<&'a String> {
    hash.get(&Yaml::String(key.to_string()))
        .and_then(|val| match val {
            Yaml::String(x) => Some(x),
            _ => None,
        })
}

fn map_doc_to_file(hash: &Hash) -> Option<String> {
    let version = get_yaml_str(hash, "apiVersion");
    let kind = get_yaml_str(hash, "kind");
    // We'll only early return if metadata.name and metadata.generatedName are absent,
    // since in that case there is nothing that uniquely identifies the object.
    let metadata = hash
        .get(&Yaml::String("metadata".to_string()))
        .and_then(|m| match m {
            Yaml::Hash(hash) => Some(hash),
            _ => None,
        })?;
    let namespace = get_yaml_str(metadata, "namespace");
    let name = get_yaml_str(metadata, "name").or_else(|| get_yaml_str(metadata, "generateName"))?;
    None
}

fn slurp(reader: &mut dyn Read) -> Result<Vec<Yaml>> {
    let mut buffer = String::new();
    reader
        .read_to_string(&mut buffer)
        .context("I/O Error: failed to read input")?;
    let vec = YamlLoader::load_from_str(&buffer).context("Failed to parse YAML input")?;
    Ok(vec)
}
