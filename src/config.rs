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
    pub git: Git<'a>,
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
pub struct Git<'a> {
    pub bg: u8,
    pub sty: Style,
    pub prefix: Cow<'a, str>,
}

impl Default for Git<'_> {
    fn default() -> Self {
        Git {
            bg: 7,
            sty: Style::color(0),
            prefix: "Git:".into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WorkDir<'a> {
    pub bg: u8,
    pub sty: Style,
    pub trun_bg: u8,
    pub trun_sty: Style,
    pub stem_bg: u8,
    pub stem_sty: Style,
    #[serde(borrow)]
    pub trun: Cow<'a, str>,
    #[serde(borrow)]
    pub dir_trun: Cow<'a, str>,
    pub max_len: usize,
    pub dir_max_len: usize,
    #[serde(borrow)]
    pub aliases: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl Default for WorkDir<'_> {
    fn default() -> Self {
        WorkDir {
            bg: 15,
            sty: Style::color(0),
            trun_bg: 15,
            trun_sty: Style::color(0),
            stem_bg: 15,
            stem_sty: Style::color(0),
            trun: "...".into(),
            dir_trun: "...".into(),
            max_len: 64,
            dir_max_len: 16,
            aliases: Default::default(),
        }
    }
}
