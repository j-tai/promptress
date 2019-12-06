use std::borrow::Cow;
use std::path::Path;

use git2::{BranchType, Error, ErrorCode, Repository, StatusOptions};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitStatus {
    pub branch: Cow<'static, str>,
    /// Number of commits ahead of remote
    pub commits_ahead: u32,
    /// Number of commits behind remote
    pub commits_behind: u32,
    /// Number of files changed in the index
    pub index_changes: u32,
    /// Number of files changed in the work tree
    pub wt_changes: u32,
    /// Number of files conflicted
    pub conflicts: u32,
}

impl GitStatus {
    pub fn is_clean_and_up_to_date(&self) -> bool {
        self.commits_ahead == 0
            && self.commits_behind == 0
            && self.index_changes == 0
            && self.wt_changes == 0
            && self.conflicts == 0
    }
}

pub fn get_status(path: &Path) -> Result<Option<GitStatus>, Error> {
    let mut s = GitStatus::default();

    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };
    let head = repo.head();
    s.branch = match head {
        Ok(h) => {
            if let Some(branch_name) = h.shorthand() {
                let branch = repo.find_branch(branch_name, BranchType::Local)?;
                if let Some(local) = branch.get().target() {
                    if let Some(upstream) = branch.upstream().ok().and_then(|b| b.get().target()) {
                        let (ahead, behind) = repo.graph_ahead_behind(local, upstream)?;
                        s.commits_ahead = ahead as u32;
                        s.commits_behind = behind as u32;
                    }
                }
                branch_name.to_string().into()
            } else {
                "--".into()
            }
        }
        Err(ref e) if e.code() == ErrorCode::UnbornBranch => "--".into(),
        Err(e) => return Err(e),
    };

    let mut opts = StatusOptions::new();
    let status = repo.statuses(Some(&mut opts))?;
    for ent in status.iter() {
        let stat = ent.status();
        if stat.is_index_new()
            || stat.is_index_modified()
            || stat.is_index_deleted()
            || stat.is_index_renamed()
            || stat.is_index_typechange()
        {
            s.index_changes += 1;
        }
        if stat.is_wt_new()
            || stat.is_wt_modified()
            || stat.is_wt_deleted()
            || stat.is_wt_renamed()
            || stat.is_wt_typechange()
        {
            s.wt_changes += 1;
        }
        if stat.is_conflicted() {
            s.conflicts += 1;
        }
    }

    Ok(Some(s))
}
