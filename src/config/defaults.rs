use super::Config;

pub struct DefaultsLoader;

impl DefaultsLoader {
    pub fn load(&self) -> Config {
        Config {
            types: vec![
                "Feat".into(),
                "Fix".into(),
                "Docs".into(),
                "Style".into(),
                "Refactor".into(),
                "Perf".into(),
                "Test".into(),
                "Build".into(),
                "CI".into(),
                "Chore".into(),
                "Revert".into(),
            ],
            scopes: vec![],
        }
    }
}
