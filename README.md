# Railroads Online Track Editor

This tool aims to enable full control track and groundwork editing for Railroads
Online.

## TODO

- [x] Save file loading
- [ ] Track Rendering
- [x] Spline types
- [ ] Better controls, such as locking the height
- [ ] Spline visibility
- [ ] Grade and Curvature calculations
- [ ] Placing new splines
- [ ] Terrain Heightmap
- [ ] Switches and crossovers

## Compiling

Since this project is in very early alpha, there are no pre-built releases. This
project requires a Rust toolchain, and potentially some native packages. The build
process is currently optimized for incremental builds, and as such requires a dynamic
library. The project doesn't actually require nightly, but the incremental builds
are faster on nightly.

## Controls

- Camera panning: Drag with the right mouse
- Camera rotation: Drag with the control key held
- Extrude: `E` while dragging a control point.

## Curves

The current system doesn't use the same type of curves as Railroads Online, primarily
because I don't actually know what type of spline the game uses. At the moment, I'm
using cubic beziers, and inserting control points between each point as an attempt
at emulating the types of splines used by the game.
