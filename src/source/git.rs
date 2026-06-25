//! Remote git source operations: clone-on-add, pull-on-resync.
//!
//! The clone is treated as a read-only content cache, so `pull` hard-resets the
//! working tree to the freshly fetched branch tip rather than attempting a merge.
//! Only public (anonymous-HTTPS) repositories are supported for now.

use std::path::Path;

use git2::{Repository, ResetType, build::RepoBuilder};

use crate::error::{Error, Result};
use crate::paths;

/// Clone if the cache is absent, otherwise fetch + fast-forward.
pub fn sync(name: &str, url: &str, branch: &str) -> Result<()> {
    let dest = paths::git_cache_dir(name);
    if dest.join(".git").is_dir() {
        tracing::info!("resyncing git source '{name}' ({branch})");
        pull(&dest, branch)
    } else {
        tracing::info!("cloning git source '{name}' from {url} ({branch})");
        clone(url, branch, &dest)
    }
}

fn clone(url: &str, branch: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }
    // A stale/partial directory would make clone fail; start fresh.
    if dest.exists() {
        std::fs::remove_dir_all(dest).map_err(|e| Error::io(dest, e))?;
    }
    RepoBuilder::new().branch(branch).clone(url, dest)?;
    Ok(())
}

fn pull(dest: &Path, branch: &str) -> Result<()> {
    let repo = Repository::open(dest)?;
    {
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[branch], None, None)?;
    }
    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let object = repo.find_object(commit.id(), None)?;
    repo.reset(&object, ResetType::Hard, None)?;
    Ok(())
}
