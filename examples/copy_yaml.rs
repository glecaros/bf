use std::collections::BTreeMap;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml_file = std::fs::File::open("examples/tasks.yaml")?;

    println!("The file exists: {}", yaml_file.metadata()?.is_file());

    let contents = std::fs::read_to_string("examples/tasks.yaml")?;
    println!("File contents: \n{}", contents);

    let yaml_string = serde_yaml::from_reader(yaml_file)?;
    println!("Read YAML string: {:#?}", yaml_string);
    Ok(())
}
