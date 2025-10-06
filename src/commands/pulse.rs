use anyhow::{Result, Context};
use colored::*;
use std::collections::HashMap;
use std::process::Command;
use indicatif::{ProgressBar, ProgressStyle};

pub fn run(rank_by_lines: bool) -> Result<()> {
    let metric = if rank_by_lines { "lines changed" } else { "commits" };
    println!("{}", format!("📊 Gathering contributor statistics by {}...", metric).bright_blue());

    // check if this is a git repo
    let status = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to check git repository status")?;
    if !status.status.success() {
        println!("{}", "❌ Not a git repository.".red());
        return Ok(());
    }

    let mut author_counts: HashMap<String, usize> = HashMap::new();
    
    if rank_by_lines {
        // Get lines changed per author
        let output = Command::new("git")
            .args(["log", "--format=%aN", "--numstat"])
            .output()
            .context("Failed to get git log output")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        
        if lines.is_empty() {
            println!("{}", "⚠️ No commits found in this repository.".yellow());
            return Ok(());
        }
        
        let bar = ProgressBar::new(lines.len() as u64);
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} processing lines")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        let mut current_author = String::new();
        
        for line in lines {
            bar.inc(1);
            
            if line.trim().is_empty() {
                continue;
            }
            
            // If line doesn't start with a digit, it's an author name
            if !line.chars().next().map_or(false, |c| c.is_numeric() || c == '-') && !line.trim().is_empty() {
                current_author = line.trim().to_string();
                continue;
            }
            
            // Parse lines added/removed
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && !current_author.is_empty() {
                let added: usize = parts[0].parse().unwrap_or(0);
                let removed: usize = parts[1].parse().unwrap_or(0);
                
                // Sum of lines added and removed
                let lines_changed = added + removed;
                *author_counts.entry(current_author.clone()).or_insert(0) += lines_changed;
            }
        }
        
        bar.finish_and_clear();
    } else {
        // Get commit counts per author (original behavior)
        let output = Command::new("git")
            .args(["log", "--pretty=%aN"])
            .output()
            .context("Failed to get git log output")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        let total = lines.len();

        if total == 0 {
            println!("{}", "⚠️ No commits found in this repository.".yellow());
            return Ok(());
        }

        // count commits per author
        let bar = ProgressBar::new(total as u64);
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} processing commits")
                .unwrap()
                .progress_chars("#>-"),
        );

        for line in lines {
            *author_counts.entry(line.to_string()).or_insert(0) += 1;
            bar.inc(1);
        }
        
        bar.finish_and_clear();
    }

    // sort by commit count descending
    let mut authors: Vec<(String, usize)> = author_counts.into_iter().collect();
    authors.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", "👥 Top contributors:\n".bright_blue());
    
    // Print table header
    println!(
        "{:<5} {:<25} {:<32} {}",
        "RANK".bold().bright_magenta(),
        "AUTHOR".bold().bright_magenta(),
        "CONTRIBUTION".bold().bright_magenta(),
        if rank_by_lines { "LINES" } else { "COMMITS" }.bold().bright_magenta()
    );
    
    // Print separator line
    println!("{}", "─".repeat(75).dimmed());
    
    // Print table rows
    for (i, (author, count)) in authors.iter().enumerate() {
        let rank = format!("{:>2}", i + 1);
        let bar_len = (*count as f64 / authors[0].1 as f64 * 30.0).round() as usize;
        let bar = "█".repeat(bar_len);
        println!(
            "{:<5} {:<25} {:<32} {}",
            rank.bright_yellow(),
            author.bright_white(),
            bar.bright_cyan(),
            format!("{}", count).dimmed()
        );
    }

    println!("\n{}", "✅ Done. Repo pulse updated.".green());

    Ok(())
}