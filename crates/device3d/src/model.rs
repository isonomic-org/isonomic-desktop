// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! A device model read from its binary model file (the published 3D
//! master: Y up, metres, plain materials, no textures), flattened into one
//! list of triangles in the device's own space.

use serde_json::Value;

use crate::math::{vec3, Transform, Vec3};

/// How a surface answers light: the model file's own numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    /// Linear RGB.
    pub base: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    /// Strength of the non-metal mirror reflection, 1 being the usual.
    pub specular: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub vertices: [u32; 3],
    pub material: u16,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Material>,
    /// The display's four corners (top left, top right, bottom left, bottom
    /// right as the person sees them) and the material it is drawn with,
    /// for a device that has a display.
    pub screen: Option<Screen>,
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct Screen {
    pub material: u16,
    pub centre: Vec3,
    /// Unit vectors across and up the display, and out of it.
    pub right: Vec3,
    pub up: Vec3,
    pub normal: Vec3,
    pub half_width: f32,
    pub half_height: f32,
}

#[derive(Debug, PartialEq)]
pub enum ModelError {
    NotAModel,
    Malformed(&'static str),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::NotAModel => f.write_str("not a binary model file"),
            ModelError::Malformed(what) => write!(f, "malformed model file: {what}"),
        }
    }
}

impl std::error::Error for ModelError {}

fn word(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn floats(value: &Value, n: usize) -> Option<Vec<f32>> {
    let list = value.as_array()?;
    (list.len() == n).then(|| {
        list.iter()
            .map(|v| v.as_f64().unwrap_or(0.) as f32)
            .collect()
    })
}

struct File<'a> {
    json: Value,
    bin: &'a [u8],
}

impl File<'_> {
    /// The bytes of accessor `index`, its component type and its count.
    fn accessor(&self, index: u64) -> Result<(&[u8], u64, usize, usize), ModelError> {
        let bad = ModelError::Malformed("accessor");
        let accessor = &self.json["accessors"][index as usize];
        let view = &self.json["bufferViews"][accessor["bufferView"].as_u64().ok_or(bad)? as usize];
        let start =
            view["byteOffset"].as_u64().unwrap_or(0) + accessor["byteOffset"].as_u64().unwrap_or(0);
        let count = accessor["count"]
            .as_u64()
            .ok_or(ModelError::Malformed("count"))? as usize;
        let kind = accessor["componentType"].as_u64().unwrap_or(0);
        let stride = view["byteStride"].as_u64().unwrap_or(0) as usize;
        let bytes = self
            .bin
            .get(start as usize..)
            .ok_or(ModelError::Malformed("buffer"))?;
        Ok((bytes, kind, count, stride))
    }

    fn vectors(&self, index: u64) -> Result<Vec<Vec3>, ModelError> {
        let (bytes, kind, count, stride) = self.accessor(index)?;
        if kind != 5126 {
            return Err(ModelError::Malformed("vector type"));
        }
        let stride = if stride == 0 { 12 } else { stride };
        let read = |at: usize| -> Option<f32> {
            Some(f32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
        };
        (0..count)
            .map(|i| {
                let at = i * stride;
                Some(vec3(read(at)?, read(at + 4)?, read(at + 8)?))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(ModelError::Malformed("vectors"))
    }

    fn indices(&self, index: u64) -> Result<Vec<u32>, ModelError> {
        let (bytes, kind, count, _) = self.accessor(index)?;
        let size = match kind {
            5121 => 1,
            5123 => 2,
            5125 => 4,
            _ => return Err(ModelError::Malformed("index type")),
        };
        (0..count)
            .map(|i| {
                let at = i * size;
                Some(match size {
                    1 => *bytes.get(at)? as u32,
                    2 => u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?) as u32,
                    _ => word(bytes, at)?,
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(ModelError::Malformed("indices"))
    }
}

fn material(value: &Value) -> Material {
    let pbr = &value["pbrMetallicRoughness"];
    let base = floats(&pbr["baseColorFactor"], 4).unwrap_or(vec![1., 1., 1., 1.]);
    let specular = &value["extensions"]["KHR_materials_specular"]["specularFactor"];
    Material {
        name: value["name"].as_str().unwrap_or("").to_string(),
        base: [base[0], base[1], base[2]],
        // The format's defaults: fully metallic, fully rough.
        metallic: pbr["metallicFactor"].as_f64().unwrap_or(1.) as f32,
        roughness: pbr["roughnessFactor"].as_f64().unwrap_or(1.) as f32,
        specular: specular.as_f64().unwrap_or(1.) as f32,
    }
}

fn node_transform(node: &Value) -> Transform {
    let rotation = floats(&node["rotation"], 4).unwrap_or(vec![0., 0., 0., 1.]);
    let shift = floats(&node["translation"], 3).unwrap_or(vec![0., 0., 0.]);
    Transform::from_parts(
        [rotation[0], rotation[1], rotation[2], rotation[3]],
        [shift[0], shift[1], shift[2]],
    )
}

impl Model {
    /// Read a binary model file.
    pub fn read(bytes: &[u8]) -> Result<Model, ModelError> {
        if bytes.get(..4) != Some(b"glTF") {
            return Err(ModelError::NotAModel);
        }
        let json_len = word(bytes, 12).ok_or(ModelError::NotAModel)? as usize;
        let json = bytes.get(20..20 + json_len).ok_or(ModelError::NotAModel)?;
        let json: Value =
            serde_json::from_slice(json).map_err(|_| ModelError::Malformed("description"))?;
        let bin_at = 20 + json_len;
        let bin_len = word(bytes, bin_at).ok_or(ModelError::Malformed("no buffer"))? as usize;
        let bin = bytes
            .get(bin_at + 8..bin_at + 8 + bin_len)
            .ok_or(ModelError::Malformed("buffer length"))?;
        let file = File { json, bin };

        let materials: Vec<Material> = file.json["materials"]
            .as_array()
            .map(|list| list.iter().map(material).collect())
            .unwrap_or_default();
        let mut model = Model {
            vertices: Vec::new(),
            triangles: Vec::new(),
            materials,
            screen: None,
            min: vec3(f32::MAX, f32::MAX, f32::MAX),
            max: vec3(f32::MIN, f32::MIN, f32::MIN),
        };
        let scene = file.json["scene"].as_u64().unwrap_or(0) as usize;
        let roots: Vec<u64> = file.json["scenes"][scene]["nodes"]
            .as_array()
            .map(|list| list.iter().filter_map(Value::as_u64).collect())
            .unwrap_or_default();
        for root in roots {
            model.add_node(&file, root, &Transform::IDENTITY, 0)?;
        }
        if model.triangles.is_empty() {
            return Err(ModelError::Malformed("no triangles"));
        }
        Ok(model)
    }

    fn add_node(
        &mut self,
        file: &File<'_>,
        index: u64,
        parent: &Transform,
        depth: usize,
    ) -> Result<(), ModelError> {
        if depth > 32 {
            return Err(ModelError::Malformed("node depth"));
        }
        let node = &file.json["nodes"][index as usize];
        let place = parent.after(&node_transform(node));
        if let Some(mesh) = node["mesh"].as_u64() {
            let is_screen = node["name"].as_str() == Some("screen");
            let primitives = file.json["meshes"][mesh as usize]["primitives"].as_array();
            for primitive in primitives.into_iter().flatten() {
                self.add_primitive(file, primitive, &place, is_screen)?;
            }
        }
        for child in node["children"].as_array().into_iter().flatten() {
            if let Some(child) = child.as_u64() {
                self.add_node(file, child, &place, depth + 1)?;
            }
        }
        Ok(())
    }

    fn add_primitive(
        &mut self,
        file: &File<'_>,
        primitive: &Value,
        place: &Transform,
        is_screen: bool,
    ) -> Result<(), ModelError> {
        let attributes = &primitive["attributes"];
        let positions = attributes["POSITION"]
            .as_u64()
            .ok_or(ModelError::Malformed("no positions"))?;
        let positions = file.vectors(positions)?;
        let normals = match attributes["NORMAL"].as_u64() {
            Some(index) => file.vectors(index)?,
            None => vec![vec3(0., 1., 0.); positions.len()],
        };
        let indices = match primitive["indices"].as_u64() {
            Some(index) => file.indices(index)?,
            None => (0..positions.len() as u32).collect(),
        };
        let material = primitive["material"].as_u64().unwrap_or(0) as u16;
        let first = self.vertices.len() as u32;
        let placed: Vec<Vec3> = positions.iter().map(|p| place.point(*p)).collect();
        for (position, normal) in placed.iter().zip(&normals) {
            self.min = self.min.min(*position);
            self.max = self.max.max(*position);
            self.vertices.push(Vertex {
                position: *position,
                normal: place.direction(*normal).normalized(),
            });
        }
        for corner in indices.as_chunks::<3>().0 {
            if corner.iter().any(|&i| i as usize >= placed.len()) {
                return Err(ModelError::Malformed("index out of range"));
            }
            self.triangles.push(Triangle {
                vertices: [first + corner[0], first + corner[1], first + corner[2]],
                material,
            });
        }
        if is_screen && !placed.is_empty() {
            self.screen = screen_of(&placed, place.direction(normals[0]).normalized(), material);
        }
        Ok(())
    }

    /// The middle of the model's bounding box.
    pub fn centre(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Paint every material whose name starts with `G_<group>` in `srgb`,
    /// as the model's colour table says a finish is applied.
    pub fn recolour(&mut self, group: &str, srgb: [u8; 3]) {
        let prefix = format!("G_{group}");
        let linear = srgb.map(|c| crate::shade::to_linear(c as f32 / 255.));
        for material in &mut self.materials {
            if material.name.starts_with(&prefix) {
                material.base = linear;
            }
        }
    }
}

/// The display's frame from its corner points: `up` is the direction in
/// its plane closest to the model's up.
fn screen_of(points: &[Vec3], normal: Vec3, material: u16) -> Option<Screen> {
    let centre = points.iter().fold(Vec3::default(), |a, p| a + *p) * (1. / points.len() as f32);
    let world_up = vec3(0., 1., 0.);
    let up = (world_up - normal * world_up.dot(normal)).normalized();
    let right = up.cross(normal).normalized();
    let extent = |axis: Vec3| {
        points
            .iter()
            .map(|p| (*p - centre).dot(axis).abs())
            .fold(0., f32::max)
    };
    let (half_width, half_height) = (extent(right), extent(up));
    (half_width > 0. && half_height > 0.).then_some(Screen {
        material,
        centre,
        right,
        up,
        normal,
        half_width,
        half_height,
    })
}
