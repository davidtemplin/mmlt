pub struct Config {
    pub scene_path: String,
    pub image_path: String,
    pub max_path_length: Option<usize>,
    pub initial_sample_count: Option<u64>,
    pub average_samples_per_pixel: Option<u64>,
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Config, String> {
        let mut scene_path: Option<String> = None;
        let mut image_path: Option<String> = None;
        let mut max_path_length: Option<usize> = None;
        let mut initial_sample_count: Option<u64> = None;
        let mut average_samples_per_pixel: Option<u64> = None;

        for chunk in args[1..].chunks(2) {
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
                "--max-path-length" => {
                    if chunk.len() != 2 {
                        return Err(String::from("no argument for --max-path-length provided"));
                    }
                    let value = &chunk[1];
                    max_path_length.replace(
                        value
                            .parse()
                            .map_err(|_| "could not parse --max-path-length value")?,
                    );
                }
                "--initial-sample-count" => {
                    if chunk.len() != 2 {
                        return Err(String::from(
                            "no argument for --initial-sample-count provided",
                        ));
                    }
                    let value = &chunk[1];
                    initial_sample_count.replace(
                        value
                            .parse()
                            .map_err(|_| "could not parse --initial-sample-count value")?,
                    );
                }
                "--average-samples-per-pixel" => {
                    if chunk.len() != 2 {
                        return Err(String::from(
                            "no argument for --average-samples-per-pixel provided",
                        ));
                    }
                    let value = &chunk[1];
                    average_samples_per_pixel.replace(
                        value
                            .parse()
                            .map_err(|_| "could not parse --average-samples-per-pixel value")?,
                    );
                }
                _ => return Err(format!("unknown flag: {}", flag)),
            };
        }

        let config = Config {
            scene_path: scene_path.ok_or("--scene is required")?,
            image_path: image_path.ok_or("--image is required")?,
            max_path_length,
            initial_sample_count,
            average_samples_per_pixel,
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
            String::from("mmlt"),
            String::from("--scene"),
            String::from(scene_path),
            String::from("--image"),
            String::from(image_path),
        ];
        let config = Config::parse(args).unwrap();
        assert_eq!(config.scene_path, String::from(scene_path));
        assert_eq!(config.image_path, String::from(image_path));
    }
}
