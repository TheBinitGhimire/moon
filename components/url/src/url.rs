use std::fmt::Display;

use crate::helper::{is_normalized_window_drive_letter, SPECIAL_SCHEMES};

#[derive(Debug, Clone)]
pub struct Url {
    pub scheme: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: UrlPath,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlPath {
    Opaque(String),
    List(Vec<String>),
}

impl Url {
    pub fn new() -> Self {
        Self {
            scheme: String::new(),
            host: None,
            port: None,
            path: UrlPath::List(Vec::new()),
            query: None,
            fragment: None,
        }
    }

    pub(crate) fn is_special(&self) -> bool {
        SPECIAL_SCHEMES.contains(&self.scheme.as_str())
    }

    pub(crate) fn has_opaque_path(&self) -> bool {
        match self.path {
            UrlPath::Opaque(_) => true,
            _ => false,
        }
    }

    pub(crate) fn shorten_path(&mut self) {
        if let UrlPath::List(path) = &mut self.path {
            if self.scheme == "file"
                && path.len() == 1
                && is_normalized_window_drive_letter(&path[0])
            {
                return;
            }

            path.pop();
        }
    }
}

impl UrlPath {
    pub(crate) fn append(&mut self, child_path: &str) {
        match self {
            UrlPath::List(path) => path.push(child_path.to_string()),
            UrlPath::Opaque(path) => path.push_str(child_path),
        };
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            UrlPath::List(list) => list.is_empty(),
            UrlPath::Opaque(path) => path.is_empty(),
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            UrlPath::List(path) => path.join("/"),
            UrlPath::Opaque(path) => path.to_string(),
        }
    }
}

impl PartialEq<&str> for UrlPath {
    fn eq(&self, other: &&str) -> bool {
        match self {
            UrlPath::List(path) => path.join("/") == *other,
            UrlPath::Opaque(path) => path == other,
        }
    }
}

impl Display for UrlPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://", self.scheme)?;
        if let Some(host) = &self.host {
            write!(f, "{}", host)?;
        }
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        write!(f, "/{}", self.path)?;
        if let Some(fragment) = &self.fragment {
            write!(f, "#{}", fragment)?;
        }
        if let Some(query) = &self.query {
            write!(f, "?{}", query)?;
        }
        Ok(())
    }
}
