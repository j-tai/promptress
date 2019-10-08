use std::borrow::Cow;
use std::env;
use std::mem;
use std::path::{Component, Path, PathBuf};

use crate::{Prompt, Style, WorkDir};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Part<'a> {
    Truncate,
    Root,
    RootStem,
    Dir(Cow<'a, str>),
    Stem(Cow<'a, str>),
}

impl<'a> Part<'a> {
    /// Counts the number of characters in this `Part`, ignoring
    /// truncation.
    fn count_chars(&self, conf: &WorkDir) -> usize {
        match self {
            Part::Truncate => conf.trun.chars().count(),
            Part::Root | Part::RootStem => 1,
            Part::Dir(s) | Part::Stem(s) => s.chars().count(),
        }
    }

    /// Counts the number of characters in this `Part`, after
    /// truncation.
    fn truncated_chars(&self, conf: &WorkDir) -> usize {
        self.count_chars(conf).min(conf.dir_max_len)
    }

    /// Returns whether this part should be truncated.
    fn should_truncate(&self, conf: &WorkDir) -> bool {
        self.count_chars(conf) > conf.dir_max_len
    }

    /// Returns the string to display for this part.
    fn as_str(&'a self, conf: &'a WorkDir) -> &'a str {
        match self {
            Part::Truncate => &conf.trun,
            Part::Root | Part::RootStem => "/",
            Part::Dir(s) | Part::Stem(s) => &s,
        }
    }

    /// Returns the style and background for this part.
    fn style_bg(&self, conf: &WorkDir) -> (Style, u8) {
        match self {
            Part::Truncate => (conf.trun_sty, conf.trun_bg),
            Part::Root | Part::Dir(_) => (conf.sty, conf.bg),
            Part::RootStem | Part::Stem(_) => (conf.stem_sty, conf.stem_bg),
        }
    }
}

/// Turn a path into a list of parts.
fn process_path<'a>(path: &'a Path, conf: &WorkDir) -> Vec<Part<'a>> {
    // List of parts, stored in reverse
    let mut parts = vec![];
    let mut total_len = 0;

    for component in path.components().rev() {
        let part = match component {
            Component::Prefix(_) => unimplemented!(),
            Component::RootDir => Part::Root,
            Component::Normal(dir) => Part::Dir(dir.to_string_lossy()),
            Component::CurDir => Part::Dir(".".into()),
            Component::ParentDir => Part::Dir("..".into()),
        };
        // Check if this component exceeds the length limit
        let part_chars = part.truncated_chars(conf);
        if total_len + 3 + part_chars > conf.max_len {
            parts.push(Part::Truncate);
            break;
        } else {
            parts.push(part);
            total_len += 3 + part_chars;
        }
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

fn apply_aliases<'a, 'b, I, P, Q>(path: &'a Path, aliases: I) -> Cow<'a, Path>
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
            let displayed_len = conf.dir_max_len - conf.dir_trun.chars().count();
            print!("{:.*}{}", displayed_len, part.as_str(conf), conf.dir_trun);
        } else {
            print!("{}", part.as_str(&p.conf.work_dir));
        }
    }
}

pub fn work_dir(p: &mut Prompt) {
    let dir: PathBuf = env::var_os("PWD")
        .map(|s| s.into())
        .unwrap_or_else(|| env::current_dir().expect("cannot get working directory"));
    let path = apply_aliases(&dir, &p.conf.work_dir.aliases);
    let parts = process_path(&path, &p.conf.work_dir);
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
        conf.dir_max_len = 12;
        assert!(part.should_truncate(&conf));
        assert_eq!(part.truncated_chars(&conf), 12);
    }

    #[test]
    fn process_path_relative() {
        let path = Path::new("user/foo");
        assert_eq!(
            process_path(path, &Default::default()),
            vec![Part::Dir("user".into()), Part::Stem("foo".into())]
        );
    }

    #[test]
    fn process_path_absolute() {
        let path = Path::new("/home/user/foo");
        assert_eq!(
            process_path(path, &Default::default()),
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
            process_path(path, &Default::default()),
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
            process_path(path, &Default::default()),
            vec![Part::RootStem]
        );
    }

    #[test]
    fn process_path_truncate() {
        let path = Path::new("/one/two/three/four/five/six/seven");
        let mut conf = WorkDir::default();
        conf.max_len = 10;
        assert_eq!(
            process_path(path, &conf),
            vec![Part::Truncate, Part::Stem("seven".into())]
        );
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
        assert_eq!(apply_aliases(path, &aliases), Path::new("/alias/foobar"));
    }
}
