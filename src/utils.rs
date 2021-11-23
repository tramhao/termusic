use pinyin::ToPinyin;
use tuirealm::tui::Terminal as TuiTerminal;

pub fn get_pin_yin(input: &str) -> String {
    let mut b = String::new();
    for (index, f) in input.to_pinyin().enumerate() {
        match f {
            Some(p) => {
                b.push_str(p.plain());
            }
            None => {
                if let Some(c) = input.to_uppercase().chars().nth(index) {
                    b.push(c);
                }
            }
        }
    }
    b
}
pub fn clear_image(&mut terminal: TuiTerminal) {
    if write!(terminal.backend_mut(), "\x1b_Ga=d\x1b\\").is_ok()
        && terminal.backend_mut().flush().is_ok()
    {}
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {

    use crate::utils::get_pin_yin;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pin_yin() {
        assert_eq!(get_pin_yin("陈一发儿"), "chenyifaer".to_string());
        assert_eq!(get_pin_yin("Gala乐队"), "GALAledui".to_string());
        assert_eq!(get_pin_yin("乐队Gala乐队"), "leduiGALAledui".to_string());
        assert_eq!(get_pin_yin("Annett Louisan"), "ANNETT LOUISAN".to_string());
    }
}
