use std::{
    fs::File,
    io::{self, Write},
};

use crate::{
    rust_template::{self, ProgramTemplate},
    solidity_template,
};

/// Test initialization template
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TestTemplate {
    /// Generate template for Jest unit-test        
    Mocha { js: bool, solidity: bool },
    /// Generate template for Jest unit-test    
    Jest { js: bool, solidity: bool },
    /// Generate template for Rust unit-test
    Rust,
}

impl TestTemplate {
    pub fn new(
        program_templates: &[ProgramTemplate],
        js: bool,
        jest: bool,
        solidity: bool,
    ) -> Self {
        if jest {
            return Self::Jest { js, solidity };
        }

        for program_template in program_templates {
            if let ProgramTemplate::RustTest = program_template {
                return Self::Rust;
            }
        }

        Self::Mocha { js, solidity }
    }

    pub fn get_test_script(&self) -> &str {
        match &self {
            Self::Mocha { js, .. } => {
                if *js {
                    "yarn run mocha -t 1000000 tests/"
                } else {
                    "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
                }
            }
            Self::Jest { js, .. } => {
                if *js {
                    "yarn run jest"
                } else {
                    "yarn run jest --preset ts-jest"
                }
            }
            Self::Rust => "cargo test",
        }
    }

    pub fn create_test_files(&self, project_name: &str) -> io::Result<()> {
        match self {
            Self::Mocha { js, solidity } => {
                if *js {
                    let mut test = File::create(format!("tests/{}.js", &project_name))?;
                    if *solidity {
                        test.write_all(solidity_template::mocha(project_name).as_bytes())?;
                    } else {
                        test.write_all(rust_template::mocha(project_name).as_bytes())?;
                    }
                } else {
                    let mut mocha = File::create(format!("tests/{}.ts", &project_name))?;
                    if *solidity {
                        mocha.write_all(solidity_template::ts_mocha(project_name).as_bytes())?;
                    } else {
                        mocha.write_all(rust_template::ts_mocha(project_name).as_bytes())?;
                    }
                }
            }
            Self::Jest { js: _, solidity } => {
                let mut test = File::create(format!("tests/{}.test.js", &project_name))?;
                if *solidity {
                    test.write_all(solidity_template::jest(project_name).as_bytes())?;
                } else {
                    test.write_all(rust_template::jest(project_name).as_bytes())?;
                }
            }
            Self::Rust => {}
        }

        Ok(())
    }
}
