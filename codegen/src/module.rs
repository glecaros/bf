use std::io::{Result, Write};

use crate::dependency_tracker::DependencyTracker;

struct Module {
    dependencies: DependencyTracker,
    code: Vec<String>,
}

impl Module {
    pub fn new() -> Module {
        Module {
            dependencies: DependencyTracker::new(),
            code: Vec::new()
        }
    }

    pub fn add_line(&mut self, line: &str, dependencies: &[&'static str]) {
        self.code.push(String::from(line));
        for dependency in dependencies.iter() {
            self.dependencies.track(dependency);
        }
    }

    pub fn write<T: Write>(&self, indentation_level: usize, writer: &mut T) -> Result<()> {
        let indentation = indentation_level * 4;
        self.dependencies.write(indentation_level, writer)?;
        for line in &self.code {
            writeln!(writer, "{:indent$}use {}", line, indent = indentation)?;
        }
        Ok(())
    }
}