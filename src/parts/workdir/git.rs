use std::borrow::Cow;
use std::path::Path;use std::fmt;
use std::fmt::{Display,Formatter};

use git2::{Repository,ErrorCode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitStatus {
    pub branch: Cow<'static, str>,
}

impl Display for GitStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.branch)
    }
}

pub fn get_status(path: &Path) -> Option<GitStatus> {
    let repo = Repository::open(path).ok()?;
    let head = repo.head();
    let branch = match head {
        Ok(h) => h
            .shorthand()
            .map(|s| s.to_string().into())
            .unwrap_or_else(|| "--".into()),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch => "--".into(),
        Err(e) => {
            eprintln!("promptress: git ({:?}): {}", path, e);
            "?".into()
        }
    };
    Some(GitStatus { branch })
}
