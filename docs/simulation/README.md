# Simulation Documentation

## Overview

This document outlines the procedural map generation simulation process, which culminates in the creation of a two-dimensional image rendered as a mesh composed of hexagonal tiles. Each hexagonal tile's color corresponds to a terrain type, informed by various factors computed during the simulation and derived from a predefined color palette.

## Mesh Structure

The mesh constitutes the foundational geometric representation of the terrain. We leverage a two-dimensional array (ndarray) for efficient storage and rapid access of hexagonal data. The `get` method facilitates type conversions and access to hex attributes.

The mesh also encapsulates pixel information for each hex, including displacement vectors and resolution parameters. These details are essential for precise positioning of hexes on the rendering canvas, ensuring that the final image is a true representation of the simulated world.

![Mesh Visualization](./mesh.png)

## Topography

The topographic aspect of the simulation determines the elevations and hydrography of the terrain. Initially, a noise algorithm provided by the OpenSimplex library generates a base texture for elevation. However, to introduce more complexity and natural features like mountain ranges, we implement additional modifications.

Coordinates undergo a cylindrical transformation to wrap the noise pattern around the map's width, creating a seamless transition at the edges.

![Elevation Texture](./elevations.png)

## Tectonic Plates

Tectonic plates are instrumental in shaping mountain ranges and oceanic trenches within the simulation. This is accomplished through the following steps:

1. **Seed Placement**: The map's initiation begins with the strategic placement of seed points that act as identifiers for individual tectonic plates. We use a custom algorithm to distribute these seeds, ensuring a minimum distance between them. This method is preferred over Poisson disk sampling, which tends to distribute points too uniformly for the desired natural variability.

   | **Custom Distribution** | **Poisson Distribution** |
   | ----------------------- | ------------------------ |
   | ![Custom Seed Placement](./custom_seeds.png) | ![Poisson Seed Placement](./poisson_seeds.png) |

2. **Grow Seeds**: Following seed placement, a growth algorithm expands each seed into a full-fledged tectonic plate using a breadth-first search mechanism. This expansion is facilitated by a specially designed random queue to model natural geological progression and form distinct tectonic plates on the map. Here plates get assigned a direction to represent its movement.

## Rendering

Rendering is the final phase, transforming simulation data into a visual representation. It involves converting mesh information into image coordinates and drawing hexes and terrain features.

To manage the demands of large image sizes, the rendering process is divided into quadrants. Each quadrant is processed in parallel, yielding a performance increase of over 300%.

*Note: The following quadrant images are for debugging purposes and illustrate the segmented rendering approach.*

- **Top Left Quadrant**
  
  ![Top Left Quadrant](./../../_debug_top_left.png)

- **Top Right Quadrant**
  
  ![Top Right Quadrant](./../../_debug_top_right.png)

- **Bottom Left Quadrant**
  
  ![Bottom Left Quadrant](./../../_debug_bottom_left.png)

- **Bottom Right Quadrant**
  
  ![Bottom Right Quadrant](./../../_debug_bottom_right.png)
