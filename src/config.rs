pub struct Config {
    pub scene_path: String,
    pub image_path: String,
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Config, String> {
        let mut scene_path: Option<String> = None;
        let mut image_path: Option<String> = None;

        for chunk in args.chunks(2) {
            let flag = &chunk[0];

            match flag.as_str() {
                "--scene" => {
                    if chunk.len() != 2 {
                        return Err(String::from("no argument for --scene provided"));
                    }
                    let value = &chunk[1];
                    scene_path.replace(value.clone());
                }
                "--image" => {
                    if chunk.len() != 2 {
                        return Err(String::from("no argument for --image provided"));
                    }
                    let value = &chunk[1];
                    image_path.replace(value.clone());
                }
                _ => return Err(format!("unknown flag: {}", flag)),
            };
        }

        let config = Config {
            scene_path: scene_path.ok_or("--scene is required")?,
            image_path: image_path.ok_or("--image is required")?,
        };

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_parse() {
        let scene_path = "/path/to/scene.yml";
        let image_path = "/path/to/image.yml";
        let args = vec![
            String::from("--scene"),
            String::from(scene_path),
            String::from("--image"),
            String::from(image_path),
        ];
        if let Ok(config) = Config::parse(args) {
            assert_eq!(config.scene_path, String::from(scene_path));
            assert_eq!(config.image_path, String::from(image_path));
        } else {
            panic!()
        }
    }
}
