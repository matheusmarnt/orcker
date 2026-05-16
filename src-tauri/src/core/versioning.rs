//! M5.4 — Git-based config versioning for compose files.
//!
//! Each project stores a mini git repo at `{project_dir}/.orcker-history/`
//! with a working directory inside. Every call to `commit_file` writes the
//! file content into the worktree and creates a commit, building a full audit
//! trail of every compose save.

use crate::core::error::AppError;
use git2::{Repository, Signature};
use std::path::Path;

/// Ensure a git repo exists at `{project_dir}/.orcker-history/`.
/// Opens an existing repo or initialises a new one.
pub fn ensure_repo(project_dir: &Path) -> Result<Repository, AppError> {
    let history_dir = project_dir.join(".orcker-history");
    if history_dir.exists() {
        Repository::open(&history_dir).map_err(|e| AppError::Internal(e.to_string()))
    } else {
        Repository::init(&history_dir).map_err(|e| AppError::Internal(e.to_string()))
    }
}

/// Write `content` to `{worktree}/{filename}` and create a commit with `message`.
/// Handles the initial-commit edge-case (empty repo has no HEAD yet).
pub fn commit_file(
    repo: &Repository,
    filename: &str,
    content: &str,
    message: &str,
) -> Result<(), AppError> {
    let sig =
        Signature::now("orcker", "orcker@local").map_err(|e| AppError::Internal(e.to_string()))?;

    let workdir = repo
        .workdir()
        .ok_or_else(|| AppError::Internal("bare repo has no workdir".into()))?;

    // Write content into the worktree
    std::fs::write(workdir.join(filename), content)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut index = repo
        .index()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    index
        .add_path(Path::new(filename))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    index
        .write()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let tree_id = index
        .write_tree()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let tree = repo
        .find_tree(tree_id)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Resolve parent commit — None for the initial commit
    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

/// Return a unified diff string comparing the parent commit to HEAD.
/// Returns an empty string when the repo has fewer than 2 commits.
pub fn diff_last_two(repo: &Repository) -> Result<String, AppError> {
    let head_commit = match repo.head().ok().and_then(|h| h.peel_to_commit().ok()) {
        Some(c) => c,
        None => return Ok(String::new()),
    };

    let parent_commit = match head_commit.parent(0).ok() {
        Some(c) => c,
        None => return Ok(String::new()),
    };

    let head_tree = head_commit
        .tree()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let parent_tree = parent_commit
        .tree()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let diff = repo
        .diff_tree_to_tree(Some(&parent_tree), Some(&head_tree), None)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut output = String::new();
    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        use std::fmt::Write;
        let origin = line.origin();
        if ['+', '-', ' '].contains(&origin) {
            let _ = write!(output, "{}", origin);
        }
        let _ = write!(
            output,
            "{}",
            std::str::from_utf8(line.content()).unwrap_or("")
        );
        true
    })
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_repo_creates_new() {
        let dir = tempdir().unwrap();
        let repo = ensure_repo(dir.path()).unwrap();
        assert!(repo.workdir().is_some());
    }

    #[test]
    fn test_commit_file_and_diff() {
        let dir = tempdir().unwrap();
        let repo = ensure_repo(dir.path()).unwrap();

        // First commit
        commit_file(&repo, "docker-compose.yml", "version: '3'\n", "initial").unwrap();

        // Diff with only one commit should be empty
        let diff = diff_last_two(&repo).unwrap();
        assert_eq!(diff, "");

        // Second commit
        commit_file(
            &repo,
            "docker-compose.yml",
            "version: '3'\nservices:\n  app:\n",
            "add services",
        )
        .unwrap();

        let diff2 = diff_last_two(&repo).unwrap();
        assert!(diff2.contains("+services:"), "diff should show added lines");
    }
}
