use crate::ui::msg::Msg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    Forward(Msg),
}

// Note: this is not actually used in tuirealm and we also dont care about it as "Msg" does not implement it
// this can be removed once the next version of tuirealm drops (currently latest version as of writing: 3.0.1)
impl PartialOrd for UserEvent {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}
