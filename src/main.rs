use clap::{Arg, Command};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Serialize, Debug)]
struct FileNode {
    name: String,
    size: u64,
    location: Option<String>,
    children: Vec<FileNode>,
}

fn build_tree(path: &str, depth: Option<usize>) -> FileNode {
    let mut root = FileNode {
        name: path.to_string(),
        size: 0,
        location: None,
        children: Vec::new(),
    };

    for entry in WalkDir::new(path).max_depth(depth.unwrap_or(usize::MAX)) {
        if let Ok(entry) = entry {
            let size = entry.metadata().unwrap().len();
            root.size += size;

            root.children.push(FileNode {
                name: entry.file_name().to_string_lossy().to_string(),
                size,
                location: Some(entry.path().to_string_lossy().to_string()),
                children: Vec::new(),
            });
        }
    }

    root
}

use std::fs;

fn save_to_json(data: &FileNode, output_path: &str) {
    let json = serde_json::to_string_pretty(data).unwrap();
    fs::write(output_path, json).unwrap();
    println!("Report saved to {}", output_path);
}
fn filter_tree(node: &mut FileNode, min_size: u64) {
    node.children.retain(|child| child.size >= min_size);
    for child in &mut node.children {
        filter_tree(child, min_size);
    }
}
fn sort_tree_by_size(node: &mut FileNode) {
    node.children.sort_by(|a, b| b.size.cmp(&a.size));
    for child in &mut node.children {
        sort_tree_by_size(child);
    }
}

fn main() {
    let matches = Command::new("Disk Inventory")
        .version("0.1.0")
        .author("Your Name")
        .about("Analyze disk usage")
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .value_name("PATH")
                .help("Path to analyze")
                .default_value("."),
        )
        .arg(
            Arg::new("depth")
                .short('d')
                .long("depth")
                .value_name("DEPTH")
                .help("Depth for directory traversal"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help("Output report to a file"),
        )
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .value_name("SIZE")
                .help("Minimum file size to include in the report"),
        )
        .get_matches();

    let path = matches.get_one::<String>("path").unwrap();
    let depth = matches
        .get_one::<String>("depth")
        .map(|d| d.parse().unwrap());
    let output = matches.get_one::<String>("output");
    let size = matches
        .get_one::<String>("size")
        .map(|d| d.parse().unwrap());

    let mut root = build_tree(path, depth);
    sort_tree_by_size(&mut root);
    filter_tree(&mut root, size.unwrap_or(1024)); // Example: filter small files

    if let Some(output_path) = output {
        save_to_json(&root, output_path);
    } else {
        println!("{:#?}", root);
    }
}
