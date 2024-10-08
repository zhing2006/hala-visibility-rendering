# hala-visibility-rendering
[![License](https://img.shields.io/badge/License-GPL3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)
[![MSRV](https://img.shields.io/badge/rustc-1.70.0+-ab6000.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

[English](README.md) | [中文](README_CN.md) | [日本語](README_JP.md) | [한국어](README_KO.md)

## Introduction

The concept of the Visibility Buffer can be traced back to 2013, when it was first introduced by Christopher A. Burns and Warren A. Hunt in their paper [The Visibility Buffer: A Cache-Friendly Approach to Deferred Shading](https://jcgt.org/published/0002/02/04/). Over the past decade, this technology has gained widespread attention and application in the industry due to its efficiency in handling complex scenes.

Today, more and more game engines and actual game projects are combining GPU Driven and Visibility Buffer technologies to enhance rendering performance and image quality. This combination allows modern graphics rendering technology to handle scene complexity and large-scale data more efficiently, effectively reducing the burden on the CPU while fully leveraging the computational power of the GPU. This project aims to implement the most basic Visibility Rendering from scratch, starting with GPU Culling and driving all subsequent rendering steps.

![Image Intro](images/intro.png)

## Development Environment Setup

Currently, the entire development environment has only been tested on the Windows platform using an RTX 4090 (due to limited equipment, further compatibility testing is not possible at this time). The development is based on `hala-gfx`, `hala-renderer`, and `hala-imgui`.

* `hala-gfx` is responsible for Vulkan calls and encapsulation.
* `hala-renderer` is responsible for reading Mesh information from glTF files and uploading it to the GPU.
* `hala-imgui` is the Rust bridge for imGUI, responsible for displaying and interacting with the user interface.

Install Rust version 1.70+; if already installed, update to the latest version using `rustup update`. Use `git clone --recursive` to pull the repository and its submodules. Compile the Debug version with `cargo build` or the Release version with `cargo build -r`.

After compilation, you can run it directly.

    ./target/(debug or release)/hala-vis-renderer -c conf/config.toml

## Rendering Process

**Note: All the following code snippets are not directly executable. Additionally, for explanatory purposes, many Shader codes have been partially "pseudo-coded" and cannot be directly compiled.**

For the specific source code, please refer to the GitHub repository: [hala-visibility-rendering](https://github.com/zhing2006/hala-visibility-rendering).

### Data Preparation

To efficiently use GPU Driven rendering, all geometric data (Mesh) is first converted into Meshlets using the [meshopt](https://crates.io/crates/meshopt) crate.

The following is a code snippet for processing a single Mesh.
```rust
  let vertex_data_adapter = unsafe {
    meshopt::VertexDataAdapter::new(
      std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * std::mem::size_of::<HalaVertex>()),
      std::mem::size_of::<HalaVertex>(),
      0,
    ).map_err(|err| HalaRendererError::new("Failed to create vertex data adapter.", Some(Box::new(err))))?
  };
  let meshlets = meshopt::clusterize::build_meshlets(
    indices.as_slice(),
    &vertex_data_adapter,
    64,   // Maximum number of vertices per Meshlet
    124,  // Maximum number of triangles per Meshlet
    0.5,  // Cone weight, mainly affecting the efficiency of backface culling
  );
  for (meshlet_index, meshlet) in meshlets.meshlets.iter().enumerate() {
    let wrapped_meshlet = meshlets.get(meshlet_index);
    let bounds = meshopt::clusterize::compute_meshlet_bounds(
      wrapped_meshlet,
      &vertex_data_adapter,
    );

    let hala_meshlet = HalaMeshlet {
      center: bounds.center,
      radius: bounds.radius,
      cone_apex: bounds.cone_apex,
      cone_axis: bounds.cone_axis,
      cone_cutoff: bounds.cone_cutoff,
      offset_of_vertices: meshlet_vertices.len() as u32,
      num_of_vertices: meshlet.vertex_count,
      offset_of_primitives: meshlet_primitives.len() as u32,
      num_of_primitives: (wrapped_meshlet.triangles.len() / 3) as u32,
      draw_index, // Save the Draw Index of this Meshlet, simplified here. In a production environment, the true Meshlet rendering queue index should be used after CPU sorting
    };

    for i in wrapped_meshlet.vertices.iter() {
      meshlet_vertices.push(*i);
    }
    for c in wrapped_meshlet.triangles.chunks(3) {
      // Since the maximum number of vertices per Meshlet is 64, the vertex index of each triangle can be stored using 8 bits
      meshlet_primitives.push((c[0] as u32) | (c[1] as u32) << 8 | (c[2] as u32) << 16);
    }
  }
```

Remember the `draw_index` mentioned above? Next, we prepare the DrawData data for indexing.

```rust
struct DrawData {
  pub object_index: u32,
  pub material_index: u32,
}

draw_data.push(DrawData {
  object_index: mesh.index,
  material_index: prim.material_index,
});
```
object_index is used to index the relevant information of the Object, such as Transform, etc. material_index is used to index the material information used for this draw, for Alpha Test and shading.

Bind the Camera data, Light data, DrawData data, and Meshlet data for the entire scene.
```rust
hala_gfx::HalaDescriptorSetLayout::new(
  Rc::clone(&logical_device),
  &[
    // Global uniform buffer, storing global information such as vp matrix, inverse vp matrix, etc.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 0,
      descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
      ...
    },
    // Camera uniform buffer, storing camera information in the scene, such as the position of each camera.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 1,
      descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
      ...
    },
    // Light uniform buffer, storing light information in the scene, such as the position of each light.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 2,
      descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
      ...
    },
    // Storage buffer for the DrawData information mentioned above.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 3,
      descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
      ...
    },
    // Storage buffer for Meshlet information.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 4,
      descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
      ...
    },
  ],
  "main_static.descriptor_set_layout",
)?,
```

Use Bindless to bind Material data, Object data, Mesh data, and Meshlet data.
```rust
hala_gfx::HalaDescriptorSetLayout::new(
  Rc::clone(&logical_device),
  &[
    // Array of uniform buffers storing Material information.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 0,
      descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
      descriptor_count: scene.materials.len() as u32,
      ...
    },
    // Array of uniform buffers storing Object information.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 1,
      descriptor_type: hala_gfx::HalaDescriptorType::UNIFORM_BUFFER,
      descriptor_count: scene.meshes.len() as u32,
      ...
    },
    // Storage buffer array for vertex information of each Mesh.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 2,
      descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
      descriptor_count: vertex_buffers.len() as u32,
      ...
    },
    // Storage buffer array for vertex information of each Meshlet.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 3,
      descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
      descriptor_count: meshlet_vertex_buffers.len() as u32,
      ...
    },
    // Storage buffer array for triangle information of each Meshlet.
    hala_gfx::HalaDescriptorSetLayoutBinding {
      binding_index: 4,
      descriptor_type: hala_gfx::HalaDescriptorType::STORAGE_BUFFER,
      descriptor_count: meshlet_primitive_buffers.len() as u32,
      ...
    },
  ],
  "main_dynamic.descriptor_set_layout",
)?,
```

Finally, use Bindless to bind Texture data.
```rust
  hala_gfx::HalaDescriptorSetLayout::new(
    Rc::clone(&logical_device),
    &[
      // Array of all textures' Images.
      hala_gfx::HalaDescriptorSetLayoutBinding {
        binding_index: 0,
        descriptor_type: hala_gfx::HalaDescriptorType::SAMPLED_IMAGE,
        descriptor_count: scene.textures.len() as u32,
        ...
      },
      // Array of all texture Samplers.
      hala_gfx::HalaDescriptorSetLayoutBinding { // All samplers in the scene.
        binding_index: 1,
        descriptor_type: hala_gfx::HalaDescriptorType::SAMPLER,
        descriptor_count: scene.textures.len() as u32,
        ...
      },
    ],
    "textures.descriptor_set_layout",
  )?,
```

At this point, all data is ready, and we can start GPU rendering. We use a single TaskDraw to render the entire scene.
```rust
// Each Task thread group has 32 threads.
let dispatch_size_x = (scene.meshlet_count + 32 - 1) / 32;
graphics_command_buffers.draw_mesh_tasks(
  index,
  dispatch_size_x,
  1,
  1,
);
```

### GPU Culling

First, backface culling and frustum culling need to be performed. Here, the Cone and Sphere data generated during the previous Meshlet calculation will be used.
```rust
  // Backface culling.
  const float3 cone_apex = mul(per_object_data.m_mtx, float4(meshlet.cone_apex, 1.0)).xyz;
  const float3 cone_axis = normalize(mul(float4(meshlet.cone_axis, 0.0), per_object_data.i_m_mtx).xyz);
  if (dot(normalize(cone_apex - camera_position), cone_axis) >= meshlet.cone_cutoff) {
    is_visible = false;
  }

  if (is_visible) {
    // Frustum culling. Due to non-uniform scaling of the object, we convert to a bounding box instead of directly using the bounding sphere for culling.
    const float3 bound_box_min = mul(per_object_data.m_mtx, float4(meshlet.bound_sphere.xyz - meshlet.bound_sphere.w, 1.0)).xyz;
    const float3 bound_box_max = mul(per_object_data.m_mtx, float4(meshlet.bound_sphere.xyz + meshlet.bound_sphere.w, 1.0)).xyz;
    if (is_box_frustum_culled(bound_box_min, bound_box_max)) {
      is_visible = false;
    }
  }
```

Next, occlusion culling is performed using a 2-Phase Occlusion Culling method.

![Image Culling](images/culling.png)

Assume that in the current frame (Frame N), there is a previous frame (Frame N-1) marked in gray, where two boxes were rendered. This depth buffer will be used in Frame N.

In Frame N, frustum culling and backface culling are first completed. In this step, polygons outside the view frustum are culled.

Next, using the depth buffer from the previous frame, the blue ellipse and square are rendered. The ellipse is rendered regardless, but the square behind it is also rendered. This situation, where objects that should not be rendered are rendered, is called a False Positive. However, if the camera does not move significantly in the next frame, False Positives generally do not persist for long.

The orange sphere and light green triangle are culled in the first phase using the depth buffer from the previous frame. All objects culled in the first phase are marked as False Negatives and are not rendered temporarily.

Before the second phase of culling, the ellipse and square that were not culled in the first phase are drawn into the depth buffer. The second phase then uses this depth buffer for culling. The orange sphere, which was culled in the first phase, is not culled under this depth buffer and is thus rendered, proving it to be a False Negative. However, the light green triangle is still occluded by the ellipse in this phase, proving it is not a False Negative and should indeed be culled.

The specific implementation is as follows.

Pass One
```HLSL
float3 aabb_min_screen, aabb_max_screen;
// Calculate the screen-space AABB. If the box intersects with the camera's near clipping plane, to_screen_aabb returns true, and occlusion culling is not needed.
if (!to_screen_aabb(g_global_uniform.vp_mtx, bound_box_min, bound_box_max, aabb_min_screen, aabb_max_screen)) {
  // If occluded, record occlusion and visibility information.
  if (is_occluded(in_hiz_image, g_push_constants.hiz_levels, g_push_constants.hiz_size, aabb_min_screen, aabb_max_screen)) {
    is_occluded_by_hiz = true;
    is_visible = false;
  }
}

// If visible (true positive) and not occluded (false positive), mark this pass as rendered with 1. Pass Two only processes Meshlets marked as 0.
out_culling_flags.Store(meshlet_index * 4, (is_visible || !is_occluded_by_hiz) ? 1 : 0);
```

Pass Two
```HLSL
const uint culling_flag = in_culling_flags.Load(meshlet_index * 4);
// If Pass One did not render this Meshlet, start occlusion culling.
if (culling_flag == 0) {
  const float3 bound_box_min = mul(per_object_data.m_mtx, float4(meshlet.bound_sphere.xyz - meshlet.bound_sphere.w, 1.0)).xyz;
  const float3 bound_box_max = mul(per_object_data.m_mtx, float4(meshlet.bound_sphere.xyz + meshlet.bound_sphere.w, 1.0)).xyz;

  // Same as the previous phase, no further explanation needed.
  float3 aabb_min_screen, aabb_max_screen;
  if (!to_screen_aabb(g_global_uniform.vp_mtx, bound_box_min, bound_box_max, aabb_min_screen, aabb_max_screen)) {
    if (is_occluded(in_hiz_image, g_push_constants.hiz_levels, g_push_constants.hiz_size, aabb_min_screen, aabb_max_screen)) {
      is_visible = false;
    } else {
      is_visible = true;
    }
  }
}

Both Pass One and Pass Two use Wave functions to count the number of visible Meshlets and then issue the corresponding number of Mesh Shader calls.
```HLSL
if (is_visible) {
  const uint index = WavePrefixCountBits(is_visible);
  ms_payload.meshlet_indices[index] = meshlet_index;
}

// One Meshlet initiates one Mesh Shader Group.
const uint visible_count = WaveActiveCountBits(is_visible);
DispatchMesh(visible_count, 1, 1);
```

The main advantage of this method is that it does not require distinguishing between Occluder and Occludee. The depth buffer from the previous frame is directly used as the Occluder, and all objects to be rendered in the current frame are treated as Occludees. Therefore, there is no need for simplified Occluder-specific meshes.

Moreover, after processing the second phase, there will be no objects that should be rendered but are culled (false negatives), completely eliminating the phenomenon of objects momentarily disappearing, which is common in UE4.

However, there will still be objects that should be culled but are not (false positives). Whether processed on the CPU or GPU, using bounding boxes for occlusion checks cannot avoid false positives. In this method, even if the bounding box is completely occluded, false positives may still occur. However, as mentioned earlier, objects rendered due to false positives are not likely to appear multiple times in consecutive frames. For example, if the camera position in Frame N+1 is the same as in Frame N, the depth buffer from the previous frame has already rendered the ellipse, and the square will be occluded in both phases, thus not being rendered.

Despite its many advantages, this method also has a drawback: culling is performed twice. Frustum culling and backface culling are performed only once, but the more time-consuming occlusion culling may be performed up to twice, which is a significant burden.

### Rendering Visibility ID

Using Mesh Shader makes rendering Visibility ID relatively simple, as you only need to write the SV_PrimitiveID data.

```HLSL
struct ToFragmentPrimitive {
  uint primitive_id: SV_PrimitiveID;
};

primitives[triangle_id].primitive_id = pack_meshlet_triangle_index(meshlet_index, triangle_id);
```

Since our Meshlet has a maximum of 124 triangles, the triangle_id only needs 7 bits, and the meshlet_index can store 25 bits. The functions for packing and unpacking the Visibility ID are as follows.

```HLSL
uint pack_meshlet_triangle_index(uint meshlet_index, uint triangle_index) {
  return ((meshlet_index & 0x1FFFFFF) << 7) | (triangle_index & 0x7F);
}

void unpack_meshlet_triangle_index(uint packed_index, out uint meshlet_index, out uint triangle_index) {
  meshlet_index = (packed_index & 0xFFFFFF80) >> 7;
  triangle_index = packed_index & 0x7F;
}
```

The final content of the Visibility Buffer is as follows:
![Image Intro](images/visibility_id.png)

### Material Classification and Sorting

For the subsequent drawing, we need to generate IndirectDraw parameters for each type of material and draw them in 64x64 tiles.

Here, we use the method from UE5, first writing different types of materials into the Depth Buffer.

```HLSL
const uint vis_id = in_vis_buffer[screen_xy];
uint meshlet_index, triangle_id;
unpack_meshlet_triangle_index(vis_id, meshlet_index, triangle_id);

const Meshlet meshlet = g_global_meshlets[meshlet_index];
const DrawData draw_data = g_draw_data[meshlet.draw_index];
output.depth = (float)draw_data.material_index / (float)CLASSIFY_DEPTH_RANGE;
```

CLASSIFY_DEPTH_RANGE is a constant that is large enough to accommodate all material types in the scene.

Next, we use a Compute Shader to classify the materials. Let's first look at the classification function.

```HLSL
// CLASSIFY_DEPTH_RANGE = CLASSIFY_NUM_OF_MATERIALS_PER_GROUP * 32
// A uint is 32 bits, so we need a group-shared memory of length CLASSIFY_NUM_OF_MATERIALS_PER_GROUP to store material information.
groupshared uint gs_material_flag[CLASSIFY_NUM_OF_MATERIALS_PER_GROUP];

void classify_pixel(in uint2 pos) {
  if (all(lessThan(pos, g_push_constants.screen_size))) {
    const float depth = in_depth_buffer.Load(pos, 0);

    // This pixel is valid and not at infinity (background).
    if (depth > 0.0) {
      const uint vis_id = in_vis_buffer.Load(pos, 0);
      uint meshlet_index, triangle_id;
      unpack_meshlet_triangle_index(id, meshlet_index, triangle_id);

      const Meshlet meshlet = g_global_meshlets[meshlet_index];
      const DrawData draw_data = g_draw_data[meshlet.draw_index];
      const uint material_index = draw_data.material_index;
      const uint index = draw_data.material_index / 32;
      const uint bit = draw_data.material_index % 32;
      uint orig;
      // Mark the material type at this position in group-shared memory.
      InterlockedOr(gs_material_flag[index], 0x1u << bit, orig);
    }
  }
}
```

The entire classification process is as follows.

```HLSL
[numthreads(CLASSIFY_THREAD_WIDTH, CLASSIFY_THREAD_WIDTH, 1)]
void main(
  uint3 group_id : SV_GroupID,
  uint3 group_thread_id : SV_GroupThreadID,
  uint3 dispatch_thread_id : SV_DispatchThreadID)
{
  // Initialize group-shared memory.
  const uint mat_chunk_index = group_thread_id.y * CLASSIFY_THREAD_WIDTH + group_thread_id.x;
  gs_material_flag[mat_chunk_index] = 0x0;

  // Group synchronization.
  GroupMemoryBarrierWithGroupSync();

  // Classify materials for pixels within a 64x64 range.
  const uint2 base_pos = group_id.xy * CLASSIFY_TILE_WIDTH + group_thread_id.xy;
  for (uint x = 0; x < 4; x++) {
    for (uint y = 0; y < 4; y++) {
      classify_pixel(base_pos + uint2(x, y) * CLASSIFY_THREAD_WIDTH);
    }
  }

  // Group synchronization.
  GroupMemoryBarrierWithGroupSync();

  // Read classification information and generate IndirectDraw data.
  uint bits = gs_material_flag[mat_chunk_index];
  if (bits != 0) {
    const uint mat_base_index = mat_chunk_index * 32;
    while (bits != 0) {
      const uint first_bit = firstbitlow(bits);
      const uint mat_index = mat_base_index + first_bit;
      bits &= ~(0x1u << first_bit);

      // An IndirectDrawArgs is 16 bytes (vertex_count, instance_count, first_vertex, first_instance).
      const uint arg_addr = mat_index * 16;
      uint store_addr = 0;
      // Increment the instance_count field to mark that this 64x64 tile needs to be drawn.
      InterlockedAdd(out_indirect_draw_arguments, arg_addr + 4, 1, store_addr);

      // Record the index of this tile to generate the corresponding quadrilateral for drawing this tile later.
      const uint tile_no = group_id.y * g_push_constants.x_size + group_id.x;
      store_addr = ((mat_index * g_push_constants.num_of_tiles) + store_addr) * 4;
      out_tile_index.Store(store_addr, tile_no);
    }
  }
}
```

With the above Material Depth, you can call IndirectDraw to draw each type of material, using Z-Test Equal to draw only the pixels covered by the material.

```rust
for material_index in 0..num_of_materials {
  graphics_command_buffers.draw_indirect(
    index,
    self.indirect_draw_buffer.as_ref(),
    material_index as u64 * std::mem::size_of::<hala_gfx::HalaIndirectDrawCommand>() as u64,
    1,
    std::mem::size_of::<hala_gfx::HalaIndirectDrawCommand>() as u32,
  );
}
```

Here, for simplicity, the number of materials is directly used as the material type. In actual use, the material type should be used.

### Rendering GBuffer

Although GBuffer is not mandatory in Visibility Rendering, considering the increasing complexity of game rendering today, to avoid repeatedly fetching triangle geometry data and material data, especially since manual calculation of partial derivatives is required in Visibility Rendering, we still perform GBuffer rendering here to separate the Geometry phase and the Lighting phase.

First, we need to restore geometric information from the Visibility Buffer.

```HLSL
// Retrieve the indices of the three vertices of a triangle using Meshlet data.
uint3 load_primitive_index(uint index, uint draw_index) {
  const uint primitive_index = g_unique_primitives[draw_index].Load(index * 4);

  const uint triangle_index0 = (primitive_index & 0xFF);
  const uint triangle_index1 = (primitive_index & 0xFF00) >> 8;
  const uint triangle_index2 = (primitive_index & 0xFF0000) >> 16;
  return uint3(triangle_index0, triangle_index1, triangle_index2);
}

const uint vis_id = in_vis_buffer[screen_xy];
uint meshlet_index, triangle_id;
unpack_meshlet_triangle_index(vis_id, meshlet_index, triangle_id);

const Meshlet meshlet = g_global_meshlets[meshlet_index];
uint triangle_index = meshlet.offset_of_primitives + triangle_id;
const uint3 tri = load_primitive_index(triangle_index, meshlet.draw_index);
```

Use the method from http://filmicworlds.com/blog/visibility-buffer-rendering-with-material-graphs/ to calculate barycentric coordinates and partial derivatives.
```HLSL
struct BaryDeriv {
  float3 lambda;
  float3 ddx;
  float3 ddy;
};

BaryDeriv calc_full_bary(float4 pt0, float4 pt1, float4 pt2, float2 pixel_ndc, float2 win_size) {
  BaryDeriv ret = (BaryDeriv)0;

  const float3 inv_w = rcp(float3(pt0.w, pt1.w, pt2.w));

  const float2 ndc0 = pt0.xy * inv_w.x;
  const float2 ndc1 = pt1.xy * inv_w.y;
  const float2 ndc2 = pt2.xy * inv_w.z;

  const float inv_det = rcp(determinant(float2x2(ndc2 - ndc1, ndc0 - ndc1)));
  ret.ddx = float3(ndc1.y - ndc2.y, ndc2.y - ndc0.y, ndc0.y - ndc1.y) * inv_det * inv_w;
  ret.ddy = float3(ndc2.x - ndc1.x, ndc0.x - ndc2.x, ndc1.x - ndc0.x) * inv_det * inv_w;
  float ddx_sum = dot(ret.ddx, float3(1, 1, 1));
  float ddy_sum = dot(ret.ddy, float3(1, 1, 1));

  const float2 delta_vec = pixel_ndc - ndc0;
  const float interp_inv_w = inv_w.x + delta_vec.x * ddx_sum + delta_vec.y * ddy_sum;
  const float interp_w = rcp(interp_inv_w);

  ret.lambda.x = interp_w * (inv_w[0] + delta_vec.x * ret.ddx.x + delta_vec.y * ret.ddy.x);
  ret.lambda.y = interp_w * (0.0      + delta_vec.x * ret.ddx.y + delta_vec.y * ret.ddy.y);
  ret.lambda.z = interp_w * (0.0      + delta_vec.x * ret.ddx.z + delta_vec.y * ret.ddy.z);

  ret.ddx *= (2.0 / win_size.x);
  ret.ddy *= (2.0 / win_size.y);
  ddx_sum *= (2.0 / win_size.x);
  ddy_sum *= (2.0 / win_size.y);

  ret.ddy *= -1.0;
  ddy_sum *= -1.0;

  const float interp_w_ddx = 1.0 / (interp_inv_w + ddx_sum);
  const float interp_w_ddy = 1.0 / (interp_inv_w + ddy_sum);

  ret.ddx = interp_w_ddx * (ret.lambda * interp_inv_w + ret.ddx) - ret.lambda;
  ret.ddy = interp_w_ddy * (ret.lambda * interp_inv_w + ret.ddy) - ret.lambda;

  return ret;
}

float3 interpolate_with_deriv(BaryDeriv deriv, float v0, float v1, float v2) {
  const float3 merged_v = float3(v0, v1, v2);
  float3 ret;
  ret.x = dot(merged_v, deriv.lambda);
  ret.y = dot(merged_v, deriv.ddx);
  ret.z = dot(merged_v, deriv.ddy);
  return ret;
}
```

The process of retrieving specific vertex data is quite lengthy, so it is omitted here. Below is the structure for the retrieved vertex data.

```HLSL
struct VertexAttributes {
  float3 position;
  float3 position_ddx;
  float3 position_ddy;
  float3 normal;
  float3 tangent;
  float2 texcoord;
  float2 texcoord_ddx;
  float2 texcoord_ddy;
};
```

Finally, we write to the GBuffer.

```HLSL
if (mtrl.base_color_map_index != INVALID_INDEX) {
  float3 base_color = SAMPLE_TEXTURE_GRAD(
    g_textures[mtrl.base_color_map_index],
    g_samplers[mtrl.base_color_map_index],
    vertex_attributes.texcoord,
    vertex_attributes.texcoord_ddx,
    vertex_attributes.texcoord_ddy
  ).rgb;
  output.albedo = float4(base_color, 1.0);
} else {
  output.albedo = float4(mtrl.base_color, 1.0);
}

output.normal = float4(vertex_attributes.normal * 0.5 + 0.5, 1.0);
```

At this point, we have obtained the complete GBuffer data.

![Image Albedo](images/albedo.png)

![Image Normal](images/normal.png)

### Lighting the World

There isn't much to discuss at this stage. We already have Albedo, Normal, and Depth. The remaining task is to restore various attributes in world space and compute lighting using BRDF. There are already many articles on this topic, so it won't be repeated here.

![Image Final](images/final.png)

## Acknowledgements

The development of this project was greatly inspired by Monsho's https://github.com/Monsho/VisibilityBuffer. I have learned a lot from it and am deeply grateful!