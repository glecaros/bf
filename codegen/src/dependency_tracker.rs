use std::io::{Result, Write};

#[derive(Debug)]
struct MultiTree {
    value: Option<&'static str>,
    children: Vec<MultiTree>,
}

impl MultiTree {
    fn new() -> MultiTree {
        MultiTree {
            value: None,
            children: Vec::new(),
        }
    }

    fn add(&mut self, path: &[&'static str]) {
        if path.len() > 0 {
            let first = path[0];
            let found = self.children.iter_mut().find(|child| {
                if let Some(value) = child.value {
                    value == first
                } else {
                    false
                }
            });
            let tail = &path[1..];
            if let Some(root) = found {
                root.add(tail);
            } else {
                let mut new_child = MultiTree::new();
                new_child.value = Some(first);
                new_child.add(tail);
                self.children.push(new_child);
            }
        }
    }

    fn to_string(&self) -> String {
        if let Some(value) = self.value {
            let mut out = String::new();
            out += value;
            out += &match self.children.len() {
                0 => String::new(),
                1 => format!("::{}", self.children[0].to_string()),
                _ => {
                    let mut inner = String::new();
                    inner += "::{";
                    inner += &self
                        .children
                        .iter()
                        .map(|child| child.to_string())
                        .collect::<Vec<String>>()
                        .join(", ");
                    inner += "}";
                    inner
                }
            };
            out
        } else {
            self.children
                .iter()
                .map(|child| child.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        }
    }
}

struct DependencyTracker {
    tree: MultiTree,
}

impl DependencyTracker {
    pub fn new() -> DependencyTracker {
        DependencyTracker {
            tree: MultiTree::new(),
        }
    }

    pub fn track(&mut self, dependency: &'static str) {
        let parts: Vec<&'static str> = dependency.split("::").collect();
        self.tree.add(&parts);
    }

    pub fn write<T: Write>(&self, indentation_level: usize, writer: &mut T) -> Result<()> {
        let indentation = indentation_level * 4;
        for child in &self.tree.children {
            writeln!(writer, "{:indent$}use {}", child.to_string(), indent = indentation)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::MultiTree;

    fn find_entry_in_children<'a>(value: &str, entry: &'a MultiTree) -> Option<&'a MultiTree> {
        entry.children.iter().find(|entry| -> bool {
            match entry.value {
                Some(contained) => contained == value,
                None => false,
            }
        })
    }

    #[test]
    fn multi_tree_should_be_properly_built() {
        let mut tree = MultiTree::new();
        tree.add(&["std", "fs", "File"]);
        tree.add(&["std", "io", "Read"]);
        tree.add(&["std", "io", "Write"]);
        tree.add(&["std", "path", "Path"]);
        tree.add(&["std", "path", "PathBuf"]);
        assert!(matches!(tree.value, None));
        assert_eq!(tree.children.len(), 1);
        let std_entry = tree.children.first().unwrap();
        assert!(matches!(std_entry.value, Some(value) if value == "std"));
        assert_eq!(std_entry.children.len(), 3);
        let fs_entry = find_entry_in_children("fs", std_entry);
        assert!(matches!(fs_entry, Some(entry) if entry.children.len() == 1));
        let file_entry = find_entry_in_children("File", fs_entry.unwrap());
        assert!(matches!(file_entry, Some(entry) if entry.children.is_empty()));
        let io_entry = find_entry_in_children("io", std_entry);
        assert!(matches!(io_entry, Some(entry) if entry.children.len() == 2));
        let read_entry = find_entry_in_children("Read", io_entry.unwrap());
        assert!(matches!(read_entry, Some(entry) if entry.children.is_empty()));
        let write_entry = find_entry_in_children("Write", io_entry.unwrap());
        assert!(matches!(write_entry, Some(entry) if entry.children.is_empty()));
        let path_entry = find_entry_in_children("path", std_entry);
        assert!(matches!(path_entry, Some(entry) if entry.children.len() == 2));
        let path_struct_entry = find_entry_in_children("Path", path_entry.unwrap());
        assert!(matches!(path_struct_entry, Some(entry) if entry.children.is_empty()));
        let pathbuf_entry = find_entry_in_children("PathBuf", path_entry.unwrap());
        assert!(matches!(pathbuf_entry, Some(entry) if entry.children.is_empty()));
    }

    #[test]
    fn multi_tree_should_serialize_correctly() {
        let mut tree = MultiTree::new();
        tree.add(&["std", "fs", "File"]);
        tree.add(&["std", "io", "Read"]);
        tree.add(&["std", "io", "Write"]);
        tree.add(&["std", "path", "Path"]);
        tree.add(&["std", "path", "PathBuf"]);
        let serialized = tree.to_string();
        assert_eq!(
            serialized,
            "std::{fs::File, io::{Read, Write}, path::{Path, PathBuf}"
        );
    }

    #[test]
    fn multi_tree_should_serialize_correctly_multiline() {
        let mut tree = MultiTree::new();
        tree.add(&["log", "error"]);
        tree.add(&["log", "info"]);
        tree.add(&["minidom", "Element"]);
        let serialized = tree.to_string();
        let (first, second) = serialized.split_once("\n").unwrap();
        assert_eq!(first, "log::{error, info}");
        assert_eq!(second, "minidom::Element");
    }
}
