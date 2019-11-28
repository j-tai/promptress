use std::borrow::Cow;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
    pub color: u8,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub strike: bool,
}

impl Style {
    pub fn color(color: u8) -> Style {
        Style {
            color,
            ..Default::default()
        }
    }

    pub fn bold(color: u8) -> Style {
        Style {
            color,
            bold: true,
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config<'a> {
    pub dollar: Dollar,
    pub exit_code: ExitCode,
    #[serde(borrow)]
    pub work_dir: WorkDir<'a>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Dollar {
    pub bg: u8,
    pub user_sty: Style,
    pub root_sty: Style,
}

impl Default for Dollar {
    fn default() -> Self {
        Dollar {
            bg: 0,
            user_sty: Style::color(15),
            root_sty: Style::bold(9),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ExitCode {
    pub success_bg: u8,
    pub success_sty: Style,
    pub failure_bg: u8,
    pub failure_sty: Style,
}

impl Default for ExitCode {
    fn default() -> Self {
        ExitCode {
            success_bg: 0,
            success_sty: Style::bold(10),
            failure_bg: 0,
            failure_sty: Style::bold(9),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WorkDir<'a> {
    /// String to display when any path component is truncated.
    #[serde(borrow)]
    pub comp_trun: Cow<'a, str>,
    /// Maximum total length of each path component.
    pub comp_max_len: usize,

    /// Maximum total length of the path.
    pub path_max_len: usize,
    /// String to display when the entire path is truncated.
    #[serde(borrow)]
    pub path_trun: Cow<'a, str>,
    /// Background color of path truncation string.
    pub path_trun_bg: u8,
    /// Foreground style of path truncation string.
    pub path_trun_sty: Style,

    /// Normal path component background color.
    pub dir_bg: u8,
    /// Normal path component foreground style.
    pub dir_sty: Style,

    /// Base path component background color.
    pub base_bg: u8,
    /// Base path component foreground style.
    pub base_sty: Style,

    /// Git options.
    pub git: WorkDirGit<'a>,

    /// List of path aliases.
    #[serde(borrow)]
    pub aliases: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl Default for WorkDir<'_> {
    fn default() -> Self {
        WorkDir {
            comp_trun: "...".into(),
            comp_max_len: 16,
            path_max_len: 64,
            path_trun: "...".into(),
            path_trun_bg: 15,
            path_trun_sty: Style::color(0),
            dir_bg: 15,
            dir_sty: Style::color(0),
            base_bg: 15,
            base_sty: Style::color(0),
            git: Default::default(),
            aliases: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WorkDirGit<'a> {
    /// Whether or not Git is enabled.
    pub enable: bool,
    /// Git branch background color.
    pub bg: u8,
    /// Git branch foreground style.
    pub sty: Style,
    /// Git branch prefix.
    pub prefix: Cow<'a, str>,
}

impl Default for WorkDirGit<'_> {
    fn default() -> Self {
        WorkDirGit {
            enable: false,
            bg: 7,
            sty: Style::color(0),
            prefix: "Git:".into(),
        }
    }
}
