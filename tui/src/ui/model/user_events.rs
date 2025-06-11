use crate::ui::msg::Msg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserEvent {
    Forward(Msg),
}
