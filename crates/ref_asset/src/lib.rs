pub mod io;

pub mod paths {
    use std::path::PathBuf;

    pub fn level_editor_player() -> PathBuf {
        let mut path = PathBuf::new();
        path.push("./storage/content/level_editor");
        path.set_extension(super::io::SCHEMA_EXTENSION);

        path
    }

    pub fn level(name: impl AsRef<str>) -> PathBuf {
        let mut path = PathBuf::new();
        path.push("./storage/content/levels");
        path.push(name.as_ref());
        path.set_extension(super::io::SCHEMA_EXTENSION);

        path
    }

    pub fn scenario(name: impl AsRef<str>) -> PathBuf {
        let mut path = PathBuf::new();
        path.push("./storage/content/scenarios");
        path.push(name.as_ref());
        path.set_extension(super::io::SCHEMA_EXTENSION);

        path
    }
}
