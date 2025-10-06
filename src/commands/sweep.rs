use anyhow::{Result, Context};
use walkdir::WalkDir;
use std::fs;
use std::io::Write;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub fn run() -> Result<()> {
    println!("{}", "🧹 Scanning for node_modules folders...".bright_blue());

    // Collect only top-level node_modules folders in a single pass
    let mut targets = Vec::new();
    for entry in WalkDir::new(".")
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() && path.file_name().map(|n| n == "node_modules").unwrap_or(false) {
            // Check if this node_modules is inside another node_modules
            let is_nested = path.ancestors()
                .skip(1) // Skip the current directory itself
                .any(|ancestor| 
                    ancestor.file_name().map(|n| n == "node_modules").unwrap_or(false)
                );
            
            if !is_nested {
                targets.push(path.to_path_buf());
            }
        }
    }

    if targets.is_empty() {
        println!("{}", "✅ No node_modules folders found.".green());
        return Ok(());
    }

    println!("{}", format!("Found {} folders to delete:", targets.len()).yellow());
    
    // Display folders as a tree
    let current_dir = std::env::current_dir()?;
    for (i, path) in targets.iter().enumerate() {
        // Try to get relative path for cleaner display
        let display_path = path.strip_prefix(&current_dir)
            .unwrap_or(path)
            .display()
            .to_string();
        
        if i == targets.len() - 1 {
            println!("└── {}", display_path.bright_white());
        } else {
            println!("├── {}", display_path.bright_white());
        }
    }
    
    // Ask for confirmation
    print!("{}", "\nDo you want to delete these folders? [y/N]: ".bright_yellow());
    std::io::stdout().flush().unwrap(); // Ensure prompt is displayed before reading input
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("{}", "Operation cancelled.".yellow());
        return Ok(());
    }
    
    // Delete deepest paths first to avoid conflicts
    targets.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    let bar = ProgressBar::new(targets.len() as u64);
    bar.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    for path in &targets {
        let display = path.display().to_string();
        bar.set_message(format!("Removing {}", display));
        if path.exists() {
            fs::remove_dir_all(path)
                .with_context(|| format!("Failed to delete {}", display))?;
        }
        bar.inc(1);
    }

    bar.finish_with_message("✨ All node_modules folders removed!");
    println!("{}", "✅ Done. Your repo is now lighter.".green());

    Ok(())
}