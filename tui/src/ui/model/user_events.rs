use crate::ui::msg::Msg;

#[derive(Debug, Clone, PartialEq)]
pub enum UserEvent {
    Forward(Msg),
}

/// We dont actually need it, tuirealm does not actually need it.
impl Eq for UserEvent {}
