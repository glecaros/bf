use std::collections::HashMap;

use regex::Regex;

use crate::{error::Error, internal_error};

pub fn interpolate(
    input_string: &str,
    variables: &HashMap<String, String>,
) -> Result<String, Error> {
    let re = Regex::new("\\{([A-Za-z][A-Za-z0-9_-]*)\\}")?;
    let mappings: Result<Vec<(Regex, String)>, Error> = re
        .captures_iter(input_string)
        .map(|cap| {
            let var_name = &cap[1];
            if let Some(value) = variables.get(var_name) {
                let regex = format!("\\{{{}\\}}", var_name);
                Ok((Regex::new(&regex)?, value.clone()))
            } else {
                Err(internal_error!("Variable {} was not provided", var_name))
            }
        })
        .collect();
    let mappings = mappings?;
    let output = mappings
        .iter()
        .fold(String::from(input_string), |s, (re, val)| {
            let out = re.replace(&s, val);
            String::from(out.to_owned())
        });
    Ok(output)
}

mod test {
    use std::collections::HashMap;

    #[test]
    fn single_variable_present_once() {
        let mut variables = HashMap::new();
        variables.insert(String::from("var"), String::from("value"));
        let output = super::interpolate("test_{var}_1", &variables);
        assert!(matches!(output, Ok(_)));
        if let Ok(output) = output {
            assert_eq!("test_value_1", &output);
        }
    }

    #[test]
    fn single_variable_present_twice() {
        let mut variables = HashMap::new();
        variables.insert(String::from("var"), String::from("value"));
        let output = super::interpolate("test_{var}_some_{var}_2", &variables);
        assert!(matches!(output, Ok(_)));
        if let Ok(output) = output {
            assert_eq!("test_value_some_value_2", &output);
        }
    }

    #[test]
    fn multiple_variables_present_once() {
        let mut variables = HashMap::new();
        variables.insert(String::from("var1"), String::from("value1"));
        variables.insert(String::from("var2"), String::from("value2"));
        let output = super::interpolate("test_{var1}_some_{var2}_2", &variables);
        assert!(matches!(output, Ok(_)));
        if let Ok(output) = output {
            assert_eq!("test_value1_some_value2_2", &output);
        }
    }

    #[test]
    fn multiple_variables_present_multiple_times() {
        let mut variables = HashMap::new();
        variables.insert(String::from("var1"), String::from("value1"));
        variables.insert(String::from("var2"), String::from("value2"));
        let output = super::interpolate("{var2}_{var1}_some_{var1}{var2}_{var1}", &variables);
        assert!(matches!(output, Ok(_)));
        if let Ok(output) = output {
            assert_eq!("value2_value1_some_value1value2_value1", &output);
        }
    }

    #[test]
    fn single_variable_not_present() {
        let variables = HashMap::new();
        let output = super::interpolate("test_{var1}_some", &variables);
        assert!(matches!(output, Err(_)));
    }
}
