A prototypical ray tracer which implements the [Multiplexed Metropolis Light Transport](https://cs.uwaterloo.ca/~thachisu/mmlt.pdf) algorithm.

An example image:

![image-25](https://github.com/user-attachments/assets/00ef7e94-c743-4efa-8183-eada71c75d53)

This example image can be rendered using

```
target/release/mmlt --scene ./scenes/scene-3.yml --image /Users/david/Desktop/image.ppm --max-path-length 10 --initial-sample-count 1000000 --average-samples-per-pixel 1024
```

To build, use

```
cargo build --release
```

Tests can be executed with

```
cargo test
```
