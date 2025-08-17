use anyhow::{anyhow, Result};
use git2::{
    Commit, Cred, CredentialType, DiffOptions, FetchOptions, Oid, PushOptions, RemoteCallbacks,
    Repository, Signature, StatusOptions,
};
use std::path::Path;

pub struct Git;

impl Git {
    /// Initialize a new git repository
    pub fn init(path: &Path) -> Result<()> {
        Repository::init(path)
            .map_err(|e| anyhow!("Failed to initialize git repository: {}", e))?;
        
        // Create .gitignore
        let gitignore = path.join(".gitignore");
        std::fs::write(gitignore, "*.tmp\n*.swp\n.DS_Store\nsessions/\n")?;
        
        Ok(())
    }

    /// Add and commit changes
    pub fn commit(path: &Path, message: &str) -> Result<()> {
        let repo = Repository::open(path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        let mut index = repo.index()
            .map_err(|e| anyhow!("Failed to get index: {}", e))?;
        
        // Add all files
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
            .map_err(|e| anyhow!("Failed to add files: {}", e))?;
        
        index.write()
            .map_err(|e| anyhow!("Failed to write index: {}", e))?;
        
        let tree_id = index.write_tree()
            .map_err(|e| anyhow!("Failed to write tree: {}", e))?;
        
        let tree = repo.find_tree(tree_id)
            .map_err(|e| anyhow!("Failed to find tree: {}", e))?;
        
        let signature = Signature::now("bunker", "bunker@localhost")
            .map_err(|e| anyhow!("Failed to create signature: {}", e))?;
        
        // Get parent commit if exists
        let parent = if let Ok(head) = repo.head() {
            if let Some(oid) = head.target() {
                repo.find_commit(oid).ok()
            } else {
                None
            }
        } else {
            None
        };
        
        let parents = if let Some(ref p) = parent {
            vec![p]
        } else {
            vec![]
        };
        
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        ).map_err(|e| anyhow!("Failed to create commit: {}", e))?;
        
        Ok(())
    }

    /// Push to remote
    pub fn push(path: &Path) -> Result<()> {
        let repo = Repository::open(path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        let mut remote = repo.find_remote("origin")
            .map_err(|e| anyhow!("No remote 'origin' configured: {}", e))?;
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Self::credentials_callback(username_from_url)
        });
        
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        remote.push(
            &["refs/heads/main:refs/heads/main"],
            Some(&mut push_options),
        ).map_err(|e| anyhow!("Failed to push: {}", e))?;
        
        Ok(())
    }

    /// Pull from remote
    pub fn pull(path: &Path) -> Result<()> {
        let repo = Repository::open(path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        let mut remote = repo.find_remote("origin")
            .map_err(|e| anyhow!("No remote 'origin' configured: {}", e))?;
        
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Self::credentials_callback(username_from_url)
        });
        
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        remote.fetch(&["main"], Some(&mut fetch_options), None)
            .map_err(|e| anyhow!("Failed to fetch: {}", e))?;
        
        // Fast-forward merge
        let fetch_head = repo.find_reference("FETCH_HEAD")
            .map_err(|e| anyhow!("Failed to find FETCH_HEAD: {}", e))?;
        
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)
            .map_err(|e| anyhow!("Failed to find fetch commit: {}", e))?;
        
        let analysis = repo.merge_analysis(&[&fetch_commit])
            .map_err(|e| anyhow!("Failed to analyze merge: {}", e))?;
        
        if analysis.0.is_fast_forward() {
            let refname = "refs/heads/main";
            let mut reference = repo.find_reference(refname)
                .map_err(|e| anyhow!("Failed to find reference: {}", e))?;
            
            reference.set_target(fetch_commit.id(), "Fast-forward")
                .map_err(|e| anyhow!("Failed to fast-forward: {}", e))?;
            
            repo.set_head(refname)
                .map_err(|e| anyhow!("Failed to set HEAD: {}", e))?;
            
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| anyhow!("Failed to checkout: {}", e))?;
        } else if analysis.0.is_normal() {
            // Would need merge - for now, error
            return Err(anyhow!("Merge required - manual intervention needed"));
        }
        
        Ok(())
    }

    /// Get status
    pub fn status(path: &Path) -> Result<Vec<String>> {
        let repo = Repository::open(path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        
        let statuses = repo.statuses(Some(&mut opts))
            .map_err(|e| anyhow!("Failed to get status: {}", e))?;
        
        let mut changes = Vec::new();
        
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let status = entry.status();
                let status_char = if status.is_wt_new() {
                    "A"
                } else if status.is_wt_modified() {
                    "M"
                } else if status.is_wt_deleted() {
                    "D"
                } else {
                    "?"
                };
                
                changes.push(format!("{} {}", status_char, path));
            }
        }
        
        Ok(changes)
    }

    /// Get history for a file
    pub fn history(path: &Path, file: &str, limit: usize) -> Result<Vec<(String, String, String)>> {
        let repo = Repository::open(path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        let mut revwalk = repo.revwalk()
            .map_err(|e| anyhow!("Failed to create revwalk: {}", e))?;
        
        revwalk.push_head()
            .map_err(|e| anyhow!("Failed to push HEAD: {}", e))?;
        
        let mut history = Vec::new();
        let mut count = 0;
        
        for oid in revwalk {
            if count >= limit {
                break;
            }
            
            let oid = oid.map_err(|e| anyhow!("Failed to get OID: {}", e))?;
            let commit = repo.find_commit(oid)
                .map_err(|e| anyhow!("Failed to find commit: {}", e))?;
            
            // Check if this commit touched the file
            if Self::commit_touches_file(&repo, &commit, file)? {
                let hash = format!("{:.8}", oid);
                let message = commit.message().unwrap_or("").to_string();
                let time = chrono::NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default();
                
                history.push((hash, time, message));
                count += 1;
            }
        }
        
        Ok(history)
    }

    /// Check if commit touches a file
    fn commit_touches_file(repo: &Repository, commit: &Commit, file: &str) -> Result<bool> {
        let tree = commit.tree()
            .map_err(|e| anyhow!("Failed to get tree: {}", e))?;
        
        if commit.parent_count() == 0 {
            // Initial commit - check if file exists
            return Ok(tree.get_path(Path::new(file)).is_ok());
        }
        
        let parent = commit.parent(0)
            .map_err(|e| anyhow!("Failed to get parent: {}", e))?;
        
        let parent_tree = parent.tree()
            .map_err(|e| anyhow!("Failed to get parent tree: {}", e))?;
        
        let mut opts = DiffOptions::new();
        opts.pathspec(file);
        
        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(&mut opts))
            .map_err(|e| anyhow!("Failed to create diff: {}", e))?;
        
        Ok(diff.deltas().len() > 0)
    }

    /// Restore file from specific commit
    pub fn restore_file(path: &Path, file: &str, commit_hash: &str) -> Result<Vec<u8>> {
        let repo = Repository::open(path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        let oid = Oid::from_str(commit_hash)
            .map_err(|e| anyhow!("Invalid commit hash: {}", e))?;
        
        let commit = repo.find_commit(oid)
            .map_err(|e| anyhow!("Failed to find commit: {}", e))?;
        
        let tree = commit.tree()
            .map_err(|e| anyhow!("Failed to get tree: {}", e))?;
        
        let entry = tree.get_path(Path::new(file))
            .map_err(|e| anyhow!("File not found in commit: {}", e))?;
        
        let object = entry.to_object(&repo)
            .map_err(|e| anyhow!("Failed to get object: {}", e))?;
        
        let blob = object.as_blob()
            .ok_or_else(|| anyhow!("Object is not a blob"))?;
        
        Ok(blob.content().to_vec())
    }

    /// Credentials callback for SSH
    fn credentials_callback(username: Option<&str>) -> Result<Cred, git2::Error> {
        if let Ok(cred) = Cred::ssh_key_from_agent(username.unwrap_or("git")) {
            return Ok(cred);
        }
        
        let home = dirs::home_dir()
            .ok_or_else(|| git2::Error::from_str("Could not find home directory"))?;
        
        let ssh_dir = home.join(".ssh");
        let private_key = ssh_dir.join("id_rsa");
        
        if private_key.exists() {
            Cred::ssh_key(
                username.unwrap_or("git"),
                None,
                &private_key,
                None,
            )
        } else {
            Err(git2::Error::from_str("No SSH key found"))
        }
    }

    /// Check if path is a git repository
    pub fn is_repo(path: &Path) -> bool {
        Repository::open(path).is_ok()
    }

    /// Add remote
    pub fn add_remote(path: &Path, url: &str) -> Result<()> {
        let repo = Repository::open(path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        repo.remote("origin", url)
            .map_err(|e| anyhow!("Failed to add remote: {}", e))?;
        
        Ok(())
    }
}