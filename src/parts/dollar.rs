use crate::Prompt;

fn is_root() -> bool {
    users::get_effective_uid() == 0
}

pub fn dollar(p: &mut Prompt) {
    p.new_part(p.conf.dollar.bg);
    if is_root() {
        p.style(p.conf.dollar.root_sty);
        print!("#");
    } else {
        p.style(p.conf.dollar.user_sty);
        print!("$");
    }
}
