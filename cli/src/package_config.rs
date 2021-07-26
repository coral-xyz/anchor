#[derive(Debug, Default)]
pub struct ProgramConfig {
    package: PackageConfig,
}

#[derive(Debug, Default)]
pub struct PackageConfig {
    name: String,
    version: String,
    description: String,
    address: String,
}
