# Simulation documentation

This document describes the steps and logic for the simulation for the map generation.
The resulting world is a 2D image represented by a mesh of hexes. Each hexagon has a color representing a part of the terrain.
The only way the hex represents the terrain type it's by its color. The colors are assigned by multiple factors resulting from the simulation, and they come from a pallette.

### Mesh

The mesh contains all the geometric data of the hexes. The hexes are storaged in an 2D ndarray for performance. There is a get method for the hexe that help with type conversions.
It also contains the pixel data of the hexes such as the displacement required to fit them into the screen, and the resolution. This helps with the possition of the hexes in the image when rendering.

![mesh](./mesh.png)

### Rendering

The rendering is in charge of creating the images from the simulation. It converts the mesh data into points on images and then draws the hexes and the terrain.
Because maps can be very large images, the rendering is done by dividing the image into quadrants and rendering each one in parallel. This improves $+300\%$ the performance

#### Top Left

![top left](./../../_debug_top_left.png)

#### Top Right

![top right](./../../_debug_top_right.png)

#### Bottom Left

![bottom left](./../../_debug_bottom_left.png)

#### Bottom Right

![bottom right](./../../_debug_bottom_right.png)
