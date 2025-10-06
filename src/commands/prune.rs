use anyhow::{Result, Context};
use colored::*;
use git2::{Repository, BranchType};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::process::Command;

pub fn run() -> Result<()> {
    println!("{}", "🌿 Checking for stale local branches...".bright_blue());

    // open current repo
    let repo = Repository::discover(".")
        .context("Not a git repository (or any of the parent directories)")?;

    // fetch remote info (silent)
    Command::new("git")
        .args(["fetch", "--prune", "--quiet"])
        .output()
        .ok();

    // list all local branches
    let branches = repo.branches(Some(BranchType::Local))?;
    let mut stale_branches = Vec::new();

    for branch_result in branches {
        let (branch, _) = branch_result?;
        let name = branch.name()?.unwrap_or("").to_string();

        if name == "main" || name == "master" {
            continue;
        }

        // check if remote branch exists
        let remote_ref = format!("refs/remotes/origin/{}", name);
        if repo.find_reference(&remote_ref).is_err() {
            stale_branches.push(name);
        }
    }

    if stale_branches.is_empty() {
        println!("{}", "✅ No stale branches found.".green());
        return Ok(());
    }
    
    // get current branch to avoid deleting it
    let current_branch = get_current_branch();
    
    println!("{}", format!("Found {} branches to delete:", stale_branches.len()).yellow());
    
    // Display branches as a tree
    for (i, branch) in stale_branches.iter().enumerate() {
        let is_current = Some(branch.as_str()) == current_branch.as_deref();
        let branch_display = if is_current {
            format!("{} (current branch)", branch).yellow().to_string()
        } else {
            branch.bright_white().to_string()
        };
        
        if i == stale_branches.len() - 1 {
            println!("└── {}", branch_display);
        } else {
            println!("├── {}", branch_display);
        }
    }
    
    // Ask for confirmation
    print!("{}", "\nDo you want to delete these branches? [y/N]: ".bright_yellow());
    std::io::stdout().flush().unwrap(); // Ensure prompt is displayed before reading input
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("{}", "Operation cancelled.".yellow());
        return Ok(());
    }

    let bar = ProgressBar::new(stale_branches.len() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}"
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    for branch in &stale_branches {
        if Some(branch.as_str()) == current_branch.as_deref() {
            bar.println(format!(
                "{} Skipping current branch '{}'",
                "⚠️".yellow(),
                branch
            ));
            bar.inc(1);
            continue;
        }

        bar.set_message(format!("Deleting {}", branch));
        Command::new("git")
            .args(["branch", "-D", branch])
            .output()
            .with_context(|| format!("Failed to delete branch {}", branch))?;
        bar.inc(1);
    }

    bar.finish_with_message("✨ All stale branches pruned!");
    println!("{}", "✅ Done. Your repo is now tidy.".green());

    Ok(())
}

fn get_current_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(name)
    } else {
        None
    }
}