# Fractal Renderer

This project is dedicated to rendering mesmerizing videos of fractals. Fractals are intricate, self-similar patterns that exhibit complexity at every scale, and creating animations of these patterns can produce stunning visual effects. This project is under development, where so far, only the mandelbrot set is covered. 

[Demo video](https://www.youtube.com/watch?v=P6HU45feY7U) created with this repository. 

## Features

- [x] Mandelbrot set
- [x] CPU processing: Parallelization
- [x] Frame system: A simple system for defining checkpoints, with coordinates, zoom level, duration, interpolation, which will define the frames of the video. 
- [ ] GPU processing: W-GPU rendering setup and Mandelbrot fragment shader.
- [ ] Interactive GUI for visualizing coordinates in the given fractal. 
- [ ] GUI tools for the frame system.

## Usage

Until this point, we have two main configuration files
- [properties.toml](properties.toml) contains editable settings for the video, as width and height of the video and framerate.
- [frames.toml](frames.toml) contains defined frames / checkpoints, that can be edited to explore other regions of the fractal. This file also contains transitions, which defines how one frame / checkpoint will transition into the next.

In the terms of building, it is recommended to build in release mode `cargo build --release`, as rendering fractals can be heavy, and the optimizations will significantly speed up the rendering process.
It is required that `FFmpeg`, `clang` and `pkg-config` are already installed on your system for being able to build. Go [here](https://github.com/zmwangx/rust-ffmpeg/wiki/Notes-on-building#dependencies)
for further instructions on how to install these dependencies. 

## Contribute

Contributions are welcomed.
