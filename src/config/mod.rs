pub mod defaults;
pub mod markdown;
pub mod yaml;

#[derive(Debug, Clone)]
pub struct Config {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
}

pub struct ConfigChain {
    yaml_path: String,
    markdown_path: String,
}

impl ConfigChain {
    pub fn new(yaml_path: &str, markdown_path: &str) -> Self {
        Self {
            yaml_path: yaml_path.into(),
            markdown_path: markdown_path.into(),
        }
    }

    pub fn load(&self) -> Config {
        let yaml = yaml::YamlLoader::new(&self.yaml_path);
        if let Some(cfg) = yaml.load() {
            return cfg;
        }

        let md = markdown::MarkdownLoader::new(&self.markdown_path);
        if let Some(cfg) = md.load() {
            return cfg;
        }

        defaults::DefaultsLoader.load()
    }
}
