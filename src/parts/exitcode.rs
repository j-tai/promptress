use std::borrow::Cow;
use std::env;

use crate::Prompt;

pub fn exit_code(p: &mut Prompt) {
    let code: Cow<str> = match env::var("PROMPTRESS_EXIT_CODE") {
        Ok(c) => c.into(),
        Err(_) => "?".into(),
    };
    if code == "0" {
        p.new_part(p.conf.exit_code.success_bg);
        p.style(p.conf.exit_code.success_sty);
        print!("{}", code);
    } else {
        p.new_part(p.conf.exit_code.failure_bg);
        p.style(p.conf.exit_code.failure_sty);
        print!("{}", code);
    }
}
