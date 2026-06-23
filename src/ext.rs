use godot::init::{ExtensionLibrary, gdextension};

struct RpgCharacterExt;

#[gdextension]
unsafe impl ExtensionLibrary for RpgCharacterExt {}
