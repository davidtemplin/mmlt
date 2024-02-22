pub struct Config {
    pub scene_path: String,
    pub image_path: String,
}

impl Config {
    pub fn parse(_args: Vec<String>) -> Result<Config, String> {
        let config = Config {
            scene_path: String::from("/Users/david/Desktop/mmlt/scenes/scene-1.yml"),
            image_path: String::from("/Users/david/Desktop/image.pfm"),
        };
        Ok(config)
    }
}
