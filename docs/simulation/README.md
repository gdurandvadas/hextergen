# Simulation Documentation

## Overview

This document outlines the procedural map generation simulation process, which culminates in the creation of a two-dimensional image rendered as a mesh composed of hexagonal tiles. Each hexagonal tile's color corresponds to a terrain type, informed by various factors computed during the simulation and derived from a predefined color palette.

## Mesh Structure

The mesh constitutes the foundational geometric representation of the terrain. I leverage a two-dimensional array (ndarray) for efficient storage and rapid access of hexagonal data. The `get` method facilitates type conversions and access to hex attributes.

The mesh also encapsulates pixel information for each hex, including displacement vectors and resolution parameters. These details are essential for precise positioning of hexes on the rendering canvas, ensuring that the final image is a true representation of the simulated world.

![Mesh Visualization](./mesh.png)

## Topography

The topographic aspect of the simulation determines the elevations and hydrography of the terrain. Initially, a noise algorithm provided by the OpenSimplex library generates a base texture for elevation, it's made more interesting by applying octaves to the noise calculation. However, to introduce more complexity and natural features, I implement additional processes to simulate tectonic plate movement and interaction.

For the noise coordenates, I'm converting the offset `x` and `y` to a cilindrical projection. This way the left and right side of the map are continuous.

![Elevation Texture](./elevations.png)

### Tectonic Plates

Tectonic plates are instrumental in shaping mountain ranges and oceanic trenches within the simulation. This is accomplished through the following steps.

#### 1. Seed Placement

The map's initiation begins with the strategic placement of seed points that act as identifiers for individual tectonic plates. I initially wanted to use a Poisson Disk approach, but the result were too uniform for my taste, so I ended upp creating a simple logic that picks the seeds randomly ensuring aminimum distance between the points.

| **Custom Distribution**                      | **Poisson Distribution**                       |
| -------------------------------------------- | ---------------------------------------------- |
| ![Custom Seed Placement](./custom_seeds.png) | ![Poisson Seed Placement](./poisson_seeds.png) |

#### 2. Plate Growth

The growth algorithm expands each seed into a full-fledged tectonic plate using a breadth-first search mechanism. This expansion is facilitated by a specially designed random queue to model natural geological progression and form distinct tectonic plates on the map. In this step, plates are also assigned a direction for the movement, which is just an random angle from $0º$ to $360º$.

#### 3. Plate Borders

To understand the relation between the plates and how they interact, we need to define their borders. This is done by going through all hexes in a plate's area and identifying if the neighbors of the hex are from the same plate or not. If they are not, we add the hex to the border list. The plates on the top and bottom edges of the map have a special edge border identifyed by the coordinate `(-1,-1)` and they are always Divergent.

![Tectonic Plates Borders](./plates_border.png)

To understand the interaction between the plates, we check the relative plate's direction between the neighbors.

> **Note**: There are a lot of interaction types for tectonic plates in the real world. They depend on the plate varian (Continental or Oceanic), direction of movement, compression rate.
> But for this simulation, I'm only focusing on interactions that bring visual apeal to the map.

| **Visualization**                                              | **Description**                                                                                                            |
| -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| ![Angle Between](./simulation/angle_between.png)               | Identify the delta between $0º$ and the angle between the plates' seeds.                                                   |
| ![Rectified Directions](./simulation/rectified_directions.png) | We rotate the angle between to become our new $0º$, which help us to understand the relative direction between the plates. |

There are four type of interactions we can expect with this simulation:
| **Directions**                   | **Interaction**                                        |
| -------------------------------- | ------------------------------------------------------ |
| ${\rightarrow} \| {\leftarrow}$  | ${\text{Convergent}}$                                  |
| ${\leftarrow}  \| {\rightarrow}$ | ${\text{Divergent}}$                                   |
| ${\rightarrow} \| {\rightarrow}$ | $A_m > B_m ? {\text{Convergent}} : {\text{Divergent}}$ |
| ${\leftarrow}  \| {\leftarrow}$  | $B_m > A_m ? {\text{Convergent}} : {\text{Divergent}}$ |

Where $A_m$ and $B_m$ are the magnitude (total hexes in area) of the plates.

#### 4. Slopes

The slopes are a list of arrays that go from the border hex of a plate to its seed. These array will affect the elevation of the hexes in the map.
The slopes are defined by an A* algorithm that goes from the seed of the hex to the border. The cost of moving from one hex to another is defined by the difference in distance between each and the border.


| $i_I$                     | $i_{II}$                    | $i_{III}$                     | $i_{IV}$                    | $i_{n}$                   |
| ------------------------- | --------------------------- | ----------------------------- | --------------------------- | ------------------------- |
| ![Slope I](./slope_I.png) | ![Slope II](./slope_II.png) | ![Slope III](./slope_III.png) | ![Slope IV](./slope_IV.png) | ![Slope n](./slope_n.png) |

#### 5. Elevation

After establishing the initial terrain through a noise algorithm, the landscape undergoes further transformation influenced by tectonic activities.

The type of interaction—whether convergent or divergent—affects each hex along a slope differently, altering its elevation with respect to its position from the seed to the border:

- **Convergent Interaction:** Characterized by plates moving towards each other, this interaction causes elevation to increase along the slope. The elevation adjustment is calculated using an exponential-like function that scales with distance from the seed, defined as:

$$
{\text{elevation} = \left( \frac{2}{1 + e^{-k \cdot (\frac{i}{n} - 0.5)}} \right) - 1}
$$

- **Divergent Interaction:** Characterized by plates moving apart from each other, this interaction results in a decrease in elevation along the slope. Similar to convergent interactions, the adjustment is based on an exponential-like function, albeit inverted:

$$
{\text{elevation} = 1 - \left( \frac{2}{1 + e^{-k \cdot (\frac{i}{n} - 0.5)}} \right)}
$$


In these formulas, $i$ is the index of the hex in the slope (starting from 0 at the seed), $n$ is the total number of hexes in the slope, and $k$ is a constant determining the steepness of the elevation change. This design ensures that elevation changes more significantly closer to the border, emphasizing the impact of tectonic interactions on topography.

The application of these formulas transforms the landscape, ensuring that elevations remain bounded between -1 and 1, thereby maintaining realistic terrain features. Through this process, the dynamic and complex nature of geological formations is captured, illustrating the significant influence of tectonic forces on the world's topography.

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
