use std::borrow::Cow;
use std::env;
use std::mem;
use std::path::{Component, Path, PathBuf};

use crate::{Prompt, Style, WorkDir};

mod git;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Part<'a> {
    Truncate,
    Root,
    RootStem,
    Dir(Cow<'a, str>),
    Stem(Cow<'a, str>),
    Git(git::GitStatus),
}

impl<'a> Part<'a> {
    /// Counts the number of characters in this `Part`, ignoring
    /// truncation.
    fn count_chars(&self, conf: &WorkDir) -> usize {
        match self {
            Part::Truncate => conf.path_trun.chars().count(),
            Part::Root | Part::RootStem => 1,
            Part::Dir(s) | Part::Stem(s) => s.chars().count(),
            Part::Git(s) => conf.git_prefix.chars().count() + format!("{}", s).chars().count(),
        }
    }

    /// Counts the number of characters in this `Part`, after
    /// truncation.
    fn truncated_chars(&self, conf: &WorkDir) -> usize {
        self.count_chars(conf).min(conf.comp_max_len)
    }

    /// Returns whether this part should be truncated.
    fn should_truncate(&self, conf: &WorkDir) -> bool {
        self.count_chars(conf) > conf.comp_max_len
    }

    /// Returns the part's content, ignoring truncation, but including
    /// prefixes.
    fn content(&'a self, conf: &'a WorkDir) -> Cow<'a, str> {
        match self {
            Part::Truncate => conf.path_trun.as_ref().into(),
            Part::Root | Part::RootStem => "/".into(),
            Part::Dir(s) | Part::Stem(s) => s.as_ref().into(),
            Part::Git(s) => format!("{}{}", conf.git_prefix, s).into(),
        }
    }

    /// Returns the style and background for this part.
    fn style_bg(&self, conf: &WorkDir) -> (Style, u8) {
        match self {
            Part::Truncate => (conf.path_trun_sty, conf.path_trun_bg),
            Part::Root | Part::Dir(_) => (conf.dir_sty, conf.dir_bg),
            Part::RootStem | Part::Stem(_) => (conf.base_sty, conf.base_bg),
            Part::Git(_) => (conf.git_sty, conf.git_bg),
        }
    }
}

/// Turn a path into a list of parts.
fn process_path<'a>(path: &'a Path, mod_path: &'a Path, conf: &WorkDir) -> Vec<Part<'a>> {
    fn normal_path_component_eq(comp: Component, value: &str) -> bool {
        if let Component::Normal(dir) = comp {
            dir == value
        } else {
            false
        }
    }

    // List of parts, stored in reverse
    let mut parts = vec![];
    let mut total_len = 0;
    let mut current_path = Some(path);

    // Tries to add a part. If the part results in the prompt being
    // too long, then `true` is returned and a `Part::Truncate` is
    // added instead. Otherwise, adds the part and returns `false`.
    let mut try_add_part = |part: Part<'a>| {
        // Check if this component exceeds the length limit
        let part_chars = part.truncated_chars(conf);
        if total_len + 3 + part_chars > conf.path_max_len {
            parts.push(Part::Truncate);
            true
        } else {
            parts.push(part);
            total_len += 3 + part_chars;
            false
        }
    };

    for component in mod_path.components().rev() {
        let full_path = current_path.unwrap();
        // Show git branch if enabled
        if conf.git && !normal_path_component_eq(component, ".git") {
            if let Some(status) = git::get_status(full_path) {
                let part = Part::Git(status);
                if try_add_part(part) {
                    break;
                }
            }
        }
        let part = match component {
            Component::Prefix(_) => unimplemented!(),
            Component::RootDir => Part::Root,
            Component::Normal(dir) => Part::Dir(dir.to_string_lossy()),
            Component::CurDir => Part::Dir(".".into()),
            Component::ParentDir => Part::Dir("..".into()),
        };
        if try_add_part(part) {
            break;
        }
        current_path = full_path.parent();
    }

    // Replace the first dir with a stem
    for part in parts.iter_mut() {
        match part {
            Part::Root => {
                *part = Part::RootStem;
                break;
            }
            Part::Dir(dir) => {
                // part: Dir("foo")
                let dir = mem::replace(dir, "".into());
                // part: Dir("")
                let stem = Part::Stem(dir);
                // stem: Stem("foo")
                mem::replace(part, stem);
                // part: Stem("foo")
                break;
            }
            _ => {}
        }
    }

    // Reverse it back to the correct order
    parts.reverse();
    parts
}

fn apply_aliases<I, P, Q>(path: &Path, aliases: I) -> Cow<Path>
where
    I: IntoIterator<Item = (P, Q)>,
    P: AsRef<str>,
    Q: AsRef<str>,
{
    let mut longest_match = 0;
    let mut new_path = None;
    for (prefix, value) in aliases {
        let prefix = prefix.as_ref();
        let value = value.as_ref();
        let len = prefix.chars().count();
        if len <= longest_match {
            continue;
        }
        let prefix_path = Path::new(prefix);
        if let Ok(path) = path.strip_prefix(prefix_path) {
            let value_path = Path::new(value);
            new_path = Some(value_path.join(path));
            longest_match = len;
        }
    }
    match new_path {
        Some(new_path) => new_path.into(),
        None => path.into(),
    }
}

fn print_parts(parts: &[Part], p: &mut Prompt) {
    for part in parts {
        let (style, bg) = part.style_bg(&p.conf.work_dir);
        p.new_part(bg);
        p.style(style);
        if part.should_truncate(&p.conf.work_dir) {
            let conf = &p.conf.work_dir;
            let displayed_len = conf.comp_max_len - conf.comp_trun.chars().count();
            print!("{:.*}{}", displayed_len, part.content(conf), conf.comp_trun);
        } else {
            print!("{}", part.content(&p.conf.work_dir));
        }
    }
}

pub fn work_dir(p: &mut Prompt) {
    let dir: PathBuf = env::var_os("PWD")
        .map(|s| s.into())
        .unwrap_or_else(|| env::current_dir().expect("cannot get working directory"));
    let mod_path = apply_aliases(&dir, &p.conf.work_dir.aliases);
    let parts = process_path(&dir, &mod_path, &p.conf.work_dir);
    print_parts(&parts, p);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn part_truncate() {
        let part = Part::Dir("0123456789abcdef".into());
        let mut conf = WorkDir::default();
        conf.comp_max_len = 12;
        assert!(part.should_truncate(&conf));
        assert_eq!(part.truncated_chars(&conf), 12);
    }

    #[test]
    fn process_path_aliased() {
        let path = Path::new("/home/user/foo");
        let path_aliased = Path::new("User/foo");
        assert_eq!(
            process_path(path, path_aliased, &Default::default()),
            vec![Part::Dir("User".into()), Part::Stem("foo".into())]
        );
    }

    #[test]
    fn process_path_absolute() {
        let path = Path::new("/home/user/foo");
        assert_eq!(
            process_path(path, path, &Default::default()),
            vec![
                Part::Root,
                Part::Dir("home".into()),
                Part::Dir("user".into()),
                Part::Stem("foo".into()),
            ]
        );
    }

    #[test]
    fn process_path_special_parts() {
        let path = Path::new("./foo/../bar");
        assert_eq!(
            process_path(path, path, &Default::default()),
            vec![
                Part::Dir(".".into()),
                Part::Dir("foo".into()),
                Part::Dir("..".into()),
                Part::Stem("bar".into()),
            ]
        );
    }

    #[test]
    fn process_path_root() {
        let path = Path::new("/");
        assert_eq!(
            process_path(path, path, &Default::default()),
            vec![Part::RootStem]
        );
    }

    #[test]
    fn process_path_truncate() {
        let path = Path::new("/one/two/three/four/five/six/seven");
        let mut conf = WorkDir::default();
        conf.path_max_len = 10;
        assert_eq!(
            process_path(path, path, &conf),
            vec![Part::Truncate, Part::Stem("seven".into())]
        );
    }

    #[cfg(unix)]
    #[test]
    fn process_path_non_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // \xff is invalid UTF-8
        let path = Path::new(OsStr::from_bytes(b"/foo/\xff"));
        let conf = WorkDir::default();
        assert_eq!(
            process_path(path, path, &conf),
            vec![
                Part::Root,
                Part::Dir("foo".into()),
                Part::Stem("\u{fffd}".into()),
            ]
        );
    }

    #[test]
    fn apply_alias_no_match() {
        let path = Path::new("/no/match");
        let aliases: HashMap<&str, &str> = HashMap::new();
        assert_eq!(apply_aliases(path, &aliases), path);
    }

    #[test]
    fn apply_alias_exact() {
        let path = Path::new("/alias/foo");
        let mut aliases = HashMap::new();
        aliases.insert("/alias/foo", "Foo");
        assert_eq!(apply_aliases(path, &aliases), Path::new("Foo"));
    }

    #[test]
    fn apply_alias_partial() {
        let path = Path::new("/alias/foo/bar");
        let mut aliases = HashMap::new();
        aliases.insert("/alias/foo", "Foo");
        assert_eq!(apply_aliases(path, &aliases), Path::new("Foo/bar"));
    }

    #[test]
    fn apply_alias_partial_component() {
        let path = Path::new("/alias/foobar");
        let mut aliases = HashMap::new();
        aliases.insert("/alias/foo", "Foo");
        assert_eq!(apply_aliases(path, &aliases), path);
    }

    #[cfg(unix)]
    #[test]
    fn apply_alias_non_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path = Path::new(OsStr::from_bytes(b"/alias/\xff/a"));
        let aliases: HashMap<&str, &str> = HashMap::new();
        assert_eq!(apply_aliases(path, &aliases), path);
    }
}
