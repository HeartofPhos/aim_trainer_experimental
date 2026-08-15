use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

fn visit_files(path: impl AsRef<Path>, f: impl Fn(&Path)) {
    let read_dir = fs::read_dir(path).unwrap();
    for dir_entry in read_dir {
        let dir_entry = dir_entry.unwrap();
        let metadata = dir_entry.metadata().unwrap();

        if metadata.is_file() {
            f(dir_entry.path().as_ref());
        }
    }
}

fn read_write_file<T: Serialize + DeserializeOwned>(path: &Path) {
    let (value, write_target) = read_file::<T>(path).unwrap();
    write_file(&write_target, &value).unwrap();
}

pub fn read_write_all<T: Serialize + DeserializeOwned>(path: impl AsRef<Path>) {
    visit_files(path, read_write_file::<T>);
}

pub(crate) const SCHEMA_EXTENSION: &str = "json";

pub fn read_file<T: DeserializeOwned>(
    path: impl AsRef<Path>,
) -> Result<(T, WriteTarget), LoadError> {
    let path = path.as_ref();

    let file = File::open(path).map_err(|_| LoadError::FileOpen {
        path: path.display().to_string(),
    })?;
    let reader = BufReader::new(file);

    let mut value: Value =
        serde_json::from_reader(reader).map_err(|err| LoadError::ToIntermediate {
            path: path.display().to_string(),
            err,
        })?;

    let ref_lookup = value_import(&mut value, path.parent())?;

    let deserialized =
        serde_json::from_value(value).map_err(|err| LoadError::FromIntermediate {
            path: path.display().to_string(),
            err,
        })?;

    Ok((
        deserialized,
        WriteTarget {
            path: path.to_path_buf(),
            ref_lookup,
        },
    ))
}

pub struct WriteTarget {
    pub path: PathBuf,
    pub ref_lookup: RefLookup,
}

pub fn write_file<T: Serialize>(target: &WriteTarget, data: &T) -> Result<(), SaveError> {
    let path = &target.path;
    let ref_lookup = &target.ref_lookup;

    let mut value = serde_json::to_value(data).map_err(|err| SaveError::ToIntermediate {
        path: path.display().to_string(),
        err,
    })?;

    export_value(&mut value, ref_lookup);

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|_| SaveError::DirectoryCreate {
            path: path.display().to_string(),
        })?;
    }

    let mut file = File::create(path).map_err(|_| SaveError::FileCreate {
        path: path.display().to_string(),
    })?;

    serde_json::to_writer_pretty(&mut file, &value).map_err(|err| SaveError::FromIntermediate {
        path: path.display().to_string(),
        err,
    })?;

    file.write_all("\n".as_bytes()).unwrap();

    Ok(())
}

#[derive(Debug)]
pub enum RefLookup {
    Value,
    Ref(String),
    Array(Vec<RefLookup>),
    Object(HashMap<String, RefLookup>),
}

const IMPORT_PREFIX: &str = "#import ";
fn value_import(value: &mut Value, root: Option<&Path>) -> Result<RefLookup, LoadError> {
    let ref_lookup = match value {
        Value::Null => RefLookup::Value,
        Value::Bool(_) => RefLookup::Value,
        Value::Number(_) => RefLookup::Value,
        Value::String(str) => {
            if str.len() > IMPORT_PREFIX.len() && str.starts_with(IMPORT_PREFIX) {
                let import_path = PathBuf::from(str[IMPORT_PREFIX.len()..str.len()].trim());

                let path = if import_path.is_relative()
                    && let Some(root) = root
                {
                    root.join(import_path)
                } else {
                    import_path
                };

                let (imported_value, _) = read_file::<Value>(path)?;
                let ref_lookup = RefLookup::Ref(str.clone());

                *value = imported_value;

                ref_lookup
            } else {
                RefLookup::Value
            }
        }
        Value::Array(values) => {
            let mut ref_lookups = Vec::new();

            for value in values {
                ref_lookups.push(value_import(value, root)?);
            }

            RefLookup::Array(ref_lookups)
        }
        Value::Object(map) => {
            let mut ref_lookup_map = HashMap::new();

            for (key, value) in map {
                ref_lookup_map.insert(key.clone(), value_import(value, root)?);
            }

            RefLookup::Object(ref_lookup_map)
        }
    };

    Ok(ref_lookup)
}

fn export_value(value: &mut Value, ref_lookup: &RefLookup) {
    match ref_lookup {
        RefLookup::Value => (),
        RefLookup::Ref(path) => *value = Value::String(path.clone()),
        RefLookup::Array(ref_values) => {
            for (i, ref_value) in ref_values.iter().enumerate() {
                if let Some(value) = value.get_mut(i) {
                    export_value(value, ref_value);
                }
            }
        }
        RefLookup::Object(ref_map) => {
            for (key, ref_value) in ref_map.iter() {
                if let Some(value) = value.get_mut(key) {
                    export_value(value, ref_value);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    FileOpen {
        path: String,
    },
    ToIntermediate {
        path: String,
        err: serde_json::Error,
    },
    FromIntermediate {
        path: String,
        err: serde_json::Error,
    },
}

impl core::error::Error for LoadError {}
impl core::fmt::Display for LoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LoadError::FileOpen { path } => write!(f, "failed to open file '{}'", path)?,
            LoadError::ToIntermediate { path, err } => {
                writeln!(f, "failed to deserialize to intermediate")?;
                writeln!(f, "'{}' {}", path, err)?;
            }
            LoadError::FromIntermediate { path, err } => {
                writeln!(f, "failed to deserialize from intermediate")?;
                writeln!(f, "'{}' {}", path, err)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum SaveError {
    DirectoryCreate {
        path: String,
    },
    FileCreate {
        path: String,
    },
    FileWrite {
        path: String,
    },
    ToIntermediate {
        path: String,
        err: serde_json::Error,
    },
    FromIntermediate {
        path: String,
        err: serde_json::Error,
    },
}

impl core::error::Error for SaveError {}
impl core::fmt::Display for SaveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SaveError::DirectoryCreate { path } => write!(f, "failed to open file '{}'", path)?,
            SaveError::FileCreate { path } => write!(f, "failed to create file '{}'", path)?,
            SaveError::FileWrite { path } => write!(f, "failed to write file '{}'", path)?,
            SaveError::ToIntermediate { path, err } => {
                writeln!(f, "failed to serialize to intermediate")?;
                writeln!(f, "'{}' {}", path, err)?;
            }
            SaveError::FromIntermediate { path, err } => {
                writeln!(f, "failed to serialize from intermediate")?;
                writeln!(f, "'{}' {}", path, err)?;
            }
        }

        Ok(())
    }
}
