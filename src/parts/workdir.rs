use std::env;
use std::path::{Component, Path, PathBuf};

use smallvec::SmallVec;

use crate::Prompt;

pub fn work_dir(p: &mut Prompt) {
    let dir: PathBuf = env::var_os("PWD")
        .map(|s| s.into())
        .unwrap_or_else(|| env::current_dir().expect("cannot get working directory"));

    // Apply aliases
    let path = {
        let mut longest_match = 0;
        let mut new_path = None;
        for (prefix, value) in &p.conf.work_dir.aliases {
            let len = prefix.chars().count();
            if len <= longest_match {
                continue;
            }
            let prefix_path = Path::new(prefix.as_ref());
            if let Ok(path) = dir.strip_prefix(prefix_path) {
                let value_path = Path::new(value.as_ref());
                new_path = Some((value_path, path));
                longest_match = len;
            }
        }
        new_path.map(|(a, b)| a.join(b)).unwrap_or(dir)
    };

    let components: SmallVec<[_; 16]> = path.components().collect();
    let total = components.len();

    // Index of first part to display (due to truncation)
    let begin = {
        let mut begin = 0;
        let mut length = 0;
        let max_len = p.conf.work_dir.max_len;
        for (index, comp) in components.iter().enumerate().rev() {
            length += 3;
            length += match comp {
                Component::Prefix(_) => unimplemented!(),
                Component::RootDir => 1,
                Component::Normal(s) => s.to_string_lossy().chars().count(),
                Component::CurDir => 1,
                Component::ParentDir => 2,
            }
            .min(max_len);
            if length > max_len {
                begin = index + 1;
                break;
            }
        }
        begin
    };

    // Display "..." if truncated
    if begin != 0 {
        p.new_part(p.conf.work_dir.trun_bg);
        p.style(p.conf.work_dir.trun_sty);
        print!("{}", p.conf.work_dir.trun);
    }

    // Length of "..." for when a directory name is truncated
    let dir_trun_len = p.conf.work_dir.dir_trun.chars().count();
    for (index, comp) in components[begin..].iter().enumerate() {
        if index == total - 1 {
            p.new_part(p.conf.work_dir.stem_bg);
            p.style(p.conf.work_dir.stem_sty);
        } else {
            p.new_part(p.conf.work_dir.bg);
            p.style(p.conf.work_dir.sty);
        }
        let s = match comp {
            Component::Prefix(_) => unimplemented!(),
            Component::RootDir => "/".into(),
            Component::Normal(s) => s.to_string_lossy(),
            Component::CurDir => ".".into(),
            Component::ParentDir => "..".into(),
        };
        let len = s.chars().count();
        if len > p.conf.work_dir.dir_max_len {
            let new_len = p
                .conf
                .work_dir
                .dir_max_len
                .checked_sub(dir_trun_len)
                .unwrap();
            let end = s.char_indices().nth(new_len).unwrap().0;
            print!("{}{}", &s[..end], p.conf.work_dir.dir_trun);
        } else {
            print!("{}", s);
        }
    }
}
