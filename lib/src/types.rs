use crate::ids::{IdConfigEditor, IdKeyGlobal, IdKeyOther};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IdKey {
    Global(IdKeyGlobal),
    Other(IdKeyOther),
}

impl From<&IdKey> for IdConfigEditor {
    fn from(value: &IdKey) -> Self {
        match *value {
            IdKey::Global(id_key_global) => IdConfigEditor::KeyGlobal(id_key_global),
            IdKey::Other(id_key_other) => IdConfigEditor::KeyOther(id_key_other),
        }
    }
}

impl From<IdKey> for IdConfigEditor {
    fn from(value: IdKey) -> Self {
        match value {
            IdKey::Global(id_key_global) => IdConfigEditor::KeyGlobal(id_key_global),
            IdKey::Other(id_key_other) => IdConfigEditor::KeyOther(id_key_other),
        }
    }
}

/// Constant strings for Unknown values
pub mod const_unknown {
    use crate::const_str;

    const_str! {
        UNKNOWN_ARTIST "Unknown Artist",
        UNKNOWN_TITLE "Unknown Title",
        UNKNOWN_ALBUM "Unknown Album",
        UNKNOWN_FILE "Unknown File",
    }
}
