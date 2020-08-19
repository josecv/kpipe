#![forbid(unsafe_code)]
extern crate yaml_rust;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Read};
use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

fn main() -> Result<()> {
    let docs = slurp(&mut io::stdin())?;
    let by_file = split_to_files(&docs);
    for (key, value) in by_file {
        let filename = format!("{}.yaml", key);
        dump(&filename, value)?;
        print!("Wrote {}", filename);
    }
    Ok(())
}

fn dump(path: &String, document: &Yaml) -> Result<()> {
    let mut dumped = String::new();
    let mut emitter = YamlEmitter::new(&mut dumped);
    emitter.dump(document)?;
    let mut file = File::create(path)?;
    file.write_all(&dumped.into_bytes()[..])?;
    Ok(())
}

fn split_to_files<'a>(docs: &'a Vec<Yaml>) -> HashMap<String, &'a Yaml> {
    docs.iter()
        .map(|y| match y {
            Yaml::Hash(hash) => {
                let key = map_doc_to_file(hash)?;
                Some((key, y))
            }
            _ => None,
        })
        .filter(|o| o.is_some())
        .map(|o| o.expect("splitting yamls failed unexpectedly"))
        .collect()
}

fn get_yaml_str(hash: &Hash, key: &str) -> Option<String> {
    hash.get(&Yaml::String(key.to_string()))
        .and_then(|val| match val {
            Yaml::String(x) => Some(x.clone()),
            _ => None,
        })
}

fn map_doc_to_file(hash: &Hash) -> Option<String> {
    let version = get_yaml_str(hash, "apiVersion").map(|x| x.replace("/", "__"));
    let kind = get_yaml_str(hash, "kind");
    // We'll only early return if metadata.name and metadata.generatedName are absent,
    // since in that case there is nothing that uniquely identifies the object.
    // This means early returning if metadata itself is absent, of course.
    let metadata = hash
        .get(&Yaml::String("metadata".to_string()))
        .and_then(|m| match m {
            Yaml::Hash(hash) => Some(hash),
            _ => None,
        })?;
    let namespace = get_yaml_str(metadata, "namespace");
    let name = get_yaml_str(metadata, "name").or_else(|| get_yaml_str(metadata, "generateName"))?;
    Some(
        [version, kind, namespace, Some(name)]
            .iter()
            .filter(|o| o.is_some())
            .map(|o| o.as_ref().expect("Getting filename failed unexpectedly"))
            .fold("".to_string(), |acc, x| match &*acc {
                "" => x.clone(),
                _ => format!("{}__{}", acc, x),
            }),
    )
}

fn slurp(reader: &mut dyn Read) -> Result<Vec<Yaml>> {
    let mut buffer = String::new();
    reader
        .read_to_string(&mut buffer)
        .context("I/O Error: failed to read input")?;
    let vec = YamlLoader::load_from_str(&buffer).context("Failed to parse YAML input")?;
    Ok(vec)
}
