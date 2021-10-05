



// #[derive(PartialEq, Eq, Hash)]
// enum Dependency {
//     StdLib(&'static str),
//     External(&'static str),
//     Internal(&'static str)
// }

// impl Dependency {
//     fn new_std(dependecy: &'static str) -> Dependency {
//         Dependency::StdLib(dependecy)
//     }
// }

// struct Dependencies {
//     dependencies: HashSet<Dependency>
// }

// impl Dependencies {
//     fn new() -> Dependencies {
//         Dependencies {
//             dependencies: HashSet::new()
//         }
//     }

//     fn add(&mut self, dependency: Dependency) {
//         self.dependencies.insert(dependency);
//     }
// }

// struct PartNode<'a> {
//     value: &'static str,
//     children: Vec<&'a PartNode<'a>>
// }

// impl<'a> PartNode<'a> {
//     fn new(value: &'static str) -> PartNode<'a> {
//         PartNode{ value: value, children: Vec::new() }
//     }

//     fn add_children(&mut self, children: &[&'static str]) {
//         self.children.iter().find(|node| node.value == children[0])
//         self.roots.iter().find(|node )
//     }
// }

// struct MultiTree<'a> {
//     roots: Vec<PartNode<'a>>
// }

// impl<'a> MultiTree<'a> {
//     fn add_parts(&mut self, parts: Vec<&'a str>) {
//         if parts.len() > 0 {
//             let root = parts[0];
//             let found = self.roots.iter().find(|node| {
//                 root == node.value
//             });
//             let a = &parts[1..];
//             match found {
//                 Some(found) => todo!(),
//                 None => todo!(),
//             }
//         }
//     }
// }

// impl ToString for Dependencies {
//     fn to_string(&self) -> String {
//         let std_lib = Vec::new();
//         for dependency in self.dependencies {
//             match dependency {
//                 Dependency::StdLib(dep) => {
//                     let parts: Vec<&str> = dep.split("::").collect();
//                     std_lib.push(parts)
//                 },
//                 Dependency::External(_) => todo!(),
//                 Dependency::Internal(_) => todo!(),
//             };
//         }
//         let asfd = "asdf::asdf";
//         let mut a = asdf.split("::");

//         String::from("asdf")
//         // let a = self.dependencies.iter().filter_map(|dep| {
//         //     if let Dependency::StdLib(lib) = dep {
//         //         Some(lib)
//         //     } else {
//         //         None
//         //     }
//         // }).();
//     }
// }

#[derive(Debug)]
struct MultiTree {
    value: Option<&'static str>,
    chidren: Vec<MultiTree> 
}

impl MultiTree {
    fn new() -> MultiTree {
        MultiTree{ value: None, chidren: Vec::new() }
    }

    fn  add(&mut self, path: &[&'static str]) {
        if path.len() > 0 {
            let first = path[0];
            let found = self.chidren.iter_mut().find(|child| {
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
                self.chidren.push(new_child);

            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::MultiTree;

    // use crate::Dependencies;

    fn find_entry_in_children<'a>(value: &str, entry: &'a MultiTree) -> Option<&'a MultiTree> {
        entry.chidren.iter().find(|entry| -> bool {
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
        assert_eq!(tree.chidren.len(), 1);
        let std_entry = tree.chidren.first().unwrap();
        assert!(matches!(std_entry.value, Some(value) if value == "std"));
        assert_eq!(std_entry.chidren.len(), 3);
        let fs_entry = find_entry_in_children("fs", std_entry);
        assert!(matches!(fs_entry, Some(entry) if entry.chidren.len() == 1));
        let file_entry = find_entry_in_children("File", fs_entry.unwrap());
        assert!(matches!(file_entry, Some(entry) if entry.chidren.is_empty()));
        let io_entry = find_entry_in_children("io", std_entry);
        assert!(matches!(io_entry, Some(entry) if entry.chidren.len() == 2));
        let read_entry = find_entry_in_children("Read", io_entry.unwrap());
        assert!(matches!(read_entry, Some(entry) if entry.chidren.is_empty()));
        let write_entry = find_entry_in_children("Write", io_entry.unwrap());
        assert!(matches!(write_entry, Some(entry) if entry.chidren.is_empty()));
        let path_entry = find_entry_in_children("path", std_entry);
        assert!(matches!(path_entry, Some(entry) if entry.chidren.len() == 2));
        println!("{:?}", tree);
        // const EXPECTED: &str = "use std::{fs::File, io::{Read, Write}, path::{Path, PathBuf}};";
        // let dependencies = Dependencies {
        //     dependencies: vec!
        // }

    }
}


