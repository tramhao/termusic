// pub mod tui_realm_stdlib_input;
// pub mod tui_realm_stdlib_progressbar;

pub mod tui_realm_stdlib_input {
    pub use tui_realm_stdlib::components::Input;
}

pub mod tui_realm_stdlib_progressbar {
    pub use tui_realm_stdlib::components::LineGauge as ProgressBar;
}
