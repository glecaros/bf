use std::io::{Result, Write};

use crate::dependency_tracker::DependencyTracker;

pub struct Module {
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
            writeln!(writer, "{:indent$}{}", " ", line, indent = indentation)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! line {
    ($mod:ident, $l:expr) => {
        $mod.add_line($l, &vec![])
    };
    ($mod:expr, $l:expr => $($dep:literal),+) => {
        $mod.add_line($l, &vec![$($dep),+])
    };
}

#[cfg(test)]
mod test {

    use super::Module;

    #[test]
    fn output_should_match_input_plus_use_stmt() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut module = Module::new();
        line!(module, "struct A {");
        line!(module, "    foo: PathBuf," => "std::path::PathBuf");
        line!(module, "    bar: Option<PathBuf>," => "std::path::PathBuf");
        line!(module, "}");
        module.write(1, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();
        const EXPECTED: &str = 
r#"    use std::path::PathBuf;
    struct A {
        foo: PathBuf,
        bar: Option<PathBuf>,
    }
"#; 
        assert_eq!(EXPECTED, output);
    }
}