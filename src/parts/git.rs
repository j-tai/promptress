use git2::{ErrorCode, Repository};

use crate::Prompt;

pub fn git(p: &mut Prompt) {
    let repo = match Repository::discover(".") {
        Ok(r) => r,
        Err(_) => return,
    };
    let head = repo.head();
    let branch = match &head {
        Ok(h) => h.shorthand().unwrap_or("--"),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch => "--",
        Err(e) => {
            eprintln!("promptress: git: {}", e);
            "?"
        }
    };
    p.new_part(p.conf.git.bg);
    p.style(p.conf.git.sty);
    print!("{}{}", p.conf.git.prefix, branch);
}
