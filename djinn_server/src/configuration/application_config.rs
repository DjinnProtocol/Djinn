use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub amount_of_threads: Option<usize>,
    pub serving_directory: Option<String>
}

impl ApplicationConfig {
    pub fn build() -> ApplicationConfig {
        let defaults = ApplicationConfig::get_defaults();
        let yaml_result = std::fs::read_to_string("config.yaml");
        let yaml = match yaml_result {
            Ok(yaml) => yaml,
            Err(_) => return defaults,
        };

        let user_config: ApplicationConfig = serde_yaml::from_str(&yaml).unwrap();
        let config = defaults.merge(user_config);

        config
    }

    fn merge(&self, other: ApplicationConfig) -> Self {
        Self {
            host: other.host.or(self.host.clone()),
            port: other.port.or(self.port),
            amount_of_threads: other.amount_of_threads.or(self.amount_of_threads),
            serving_directory: other.serving_directory.or(self.serving_directory.clone())
        }
    }

    fn get_defaults() -> ApplicationConfig {
        ApplicationConfig {
            host: Some("0.0.0.0".to_string()),
            port: Some(7777),
            amount_of_threads: Some(4),
            serving_directory: Some("./files".to_string())
        }
    }
}
