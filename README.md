# Raytracer_rs

A simple [ray tracer][ray_wiki] written in rust.

The code for the ray tracing algorithm is found in [raytrace-lib](./raytrace-lib/).

To specify scenes the program uses a custom DSL parsed by the
module [scene_parser](./scene-parser/).
Some examples of this are found in the [scenes](./scenes/) folder.

[ray_wiki]: https://en.wikipedia.org/wiki/Ray_tracing_(graphics)

## Usage

Running on an example with 8 threads:

```sh
cargo build --release
./target/release/raytrace-rs -f scenes/room.scene -r 50 -o raytraced.png -p -n 8
```

See `raytrace-rs -h` for all available options.

## Images generated

<img src="./scenes/room.png" alt="Simple room" width="512"/>

## TODO

- [ ] Supersampling/Antialiasing
- [ ] More primitives
  - [ ] Cone
  - [ ] Cylinder
  - [ ] Cube
- [x] Parallelization
- [ ] Optimizations
  - [ ] Spatial Structures (grouping objects to reduce ray intersection calculations)
- [ ] [Photon mapping](https://en.wikipedia.org/wiki/Photon_mapping)
