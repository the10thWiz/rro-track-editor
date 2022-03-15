# Railroads Online Track Editor

This tool aims to enable full control track and groundwork editing for Railroads
Online.

## TODO

- [x] Save file loading
- [x] Track Rendering (Still needs better track model)
- [x] Spline types
- [ ] Better controls
    - [x] Lock Z
    - [ ] Lock slope
    - [ ] Snapping
- [x] Spline visibility
- [ ] Grade and Curvature calculations
- [ ] Placing new splines
- [ ] Terrain Heightmap
    - [ ] Industry locations
- [ ] Switches and crossovers

At the moment, I have no plans to include the ability to place or edit
the industries, or even the other buildings. I also have no intention
of adding the ability to edit the vegetation.

Railroads Online does not (yet) provide the ability to edit the terrain.

## Compiling

Since this project is in very early alpha, there are no pre-built releases. This
project requires a Rust toolchain, and potentially some native packages. The build
process is currently optimized for incremental builds, and as such requires a dynamic
library. The project doesn't actually require nightly, but the incremental builds
are faster on nightly.

## Controls

- Camera panning: Drag with the right mouse
- Camera rotation: Drag with the control key held

## Curves

The current system doesn't use the same type of curves as Railroads Online, primarily
because I don't actually know what type of spline the game uses. At the moment, I'm
using cubic beziers, and inserting control points between each point as an attempt
at emulating the types of splines used by the game.
