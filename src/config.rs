pub struct Config {
    pub scene_path: String,
    pub image_path: String,
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Config, String> {
        let mut scene_path: Option<String> = None;
        let mut image_path: Option<String> = None;

        for chunk in args.chunks(2) {
            let flag = &chunk[1];
            let value = &chunk[2];

            match flag.as_str() {
                "--scene" => scene_path.replace(value.clone()),
                "--image" => image_path.replace(value.clone()),
                _ => return Err(format!("unknown flag: {}", value)),
            };
        }

        let config = Config {
            scene_path: scene_path.ok_or("--scene is required")?,
            image_path: image_path.ok_or("--image is required")?,
        };

        Ok(config)
    }
}
