use std::borrow::Cow::Owned;
use std::collections::BTreeMap;
use std::default::Default;
use std::mem;

use crate::apis::tile::glb::private::GlbGenPrivateMethod;
use anyhow::anyhow;
use dashmap::DashMap;
use gltf::binary::Header;
use gltf::buffer::Target::{ArrayBuffer, ElementArrayBuffer};
use gltf::json::accessor::{ComponentType, GenericComponentType, Type};
use gltf::json::buffer::{Stride, View};
use gltf::json::material::{PbrBaseColorFactor, PbrMetallicRoughness};
use gltf::json::mesh::Primitive;
use gltf::json::validation::Checked::Valid;
use gltf::json::validation::USize64;
use gltf::json::{Accessor, Buffer, Material, Mesh, Node, Root, Scene, Value};
use gltf::mesh::Mode;
/// [`gltf::Glb`]に[`VMesh`]からインスタンスを生成するメソッドを追加しています。
pub use gltf::Glb;
use gltf::Semantic;
use num::cast::AsPrimitive;
use rustc_hash::FxBuildHasher;
use vec_x::VecX;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Vertex([f32; 3]);

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct UV([f32; 2]);

/// テクスチャのMIMEタイプを表す列挙型です。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mime {
    ImageJpeg,
    ImagePng,
}

/// テクスチャ情報を表す構造体です。
/// `buf`と`uri`のどちらか一方を指定してください。
/// どちらも指定した場合、出力されるglbファイルにはどちらも書き込まれますが、glbファイルの仕様上`buf`に格納した情報が優先されます。
/// どちらも指定されない場合の仕様上の動作はgLTFの規格上未定義です。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureInfo {
    pub buf: Option<Vec<u8>>,
    pub uri: Option<String>,
    pub mime_type: Mime,
}

mod private {
    use std::mem;
    use vec_x::VecX;

    pub trait GlbGenPrivateMethod {
        fn liner_rgb_to_srgb(color: VecX<u8, 3>) -> [f32; 4]
        where
        {
            let color = color.as_::<f32>();
            let max: f32 = u8::MAX as f32;

            let r = color[0] / max;
            let g = color[1] / max;
            let b = color[2] / max;
            let a = 1.;

            [r, g, b, a]
        }
        fn convert_to_byte_vec<T>(vec: Vec<T>) -> Vec<u8> {
            let byte_length = vec.len() * mem::size_of::<T>();
            let alloc = vec.into_boxed_slice();
            let ptr = Box::<[T]>::into_raw(alloc) as *mut u8;
            //　`Vec::into_boxed_slice`によって、余分な容量を破棄しているので、`byte_capacity`は`byte_length`と同じ
            unsafe { Vec::from_raw_parts(ptr, byte_length, byte_length) }
        }

        // 要素数が4の倍数になるようにdefault値で埋める
        fn pad_to_mul_of_four<T: Default + Clone>(mut vec: Vec<T>) -> Vec<T> {
            let remainder = vec.len() % 4;

            if remainder != 0 {
                vec.append(&mut vec![T::default(); 4 - remainder])
            }

            vec.shrink_to_fit();

            vec
        }

        // n以上の最小の4の倍数に切り上げる
        // bit演算は使わない
        fn round_up_to_mul_of_four(n: usize) -> usize {
            let remainder = n % 4;
            if remainder == 0 {
                n
            } else {
                n + 4 - remainder
            }
        }
    }
}


pub type Point3D = VecX<f32, 3>;
pub type Color = VecX<u8, 3>;
#[derive(Default, Debug, Clone)]
pub struct VMesh {
    pub(crate) bounds: (Point3D, Point3D),
    pub(crate) offset: Point3D,
    pub(crate) points: Vec<Point3D>,
    pub(crate) faces: DashMap<Color, Vec<usize>, FxBuildHasher>,
}

impl VMesh {
    pub fn create_water_surface(points: [Point3D; 4]) -> Self {
        let mut points = points.to_vec();
        let mut faces = DashMap::from_iter(vec![(Color::new([0, 0, 255]), vec![0, 1, 2, 2, 3, 0])]);
        let bounds = Self::calc_aabb(points.iter().cloned().collect());
        let offset = Point3D::default();
        let resolution = 0.0;

        VMesh {
            bounds,
            offset,
            points,
            faces,
        }
    }

    fn calc_aabb(points: Vec<Point3D>) -> (Point3D, Point3D) {
        let mut min = Point3D::from([f32::MAX; 3]);
        let mut max = Point3D::from([f32::MIN; 3]);
        for point in points.iter() {
            for i in 0..3 {
                min[i] = min[i].min(point[i]);
                max[i] = max[i].max(point[i]);
            }
        }
        (min, max)
    }
}

pub trait WaterGlbGen<'a>: private::GlbGenPrivateMethod {
    /// ボクセルメッシュから[`Glb`]のインスタンスを生成します。
    fn from_vmesh(vmesh: VMesh) -> Result<Glb<'a>, anyhow::Error>
    {
        let mut root = Root::default();

        let vertices = vmesh.points.into_iter().map(|point| {
            let [x, y, z] = point.as_().data;
            // gltfの座標系に合わせる
            Vertex([x, z, -y])
        }).collect::<Vec<_>>();

        let (colors, indices): (Vec<_>, Vec<_>) = vmesh.faces.into_iter().map(|(color, vertex_ids)| {
            let color = Self::liner_rgb_to_srgb(color);

            let vertex_ids = vertex_ids.into_iter().map(|vertex_id| {
                vertex_id as u32
            }).collect::<Vec<_>>();

            (color, vertex_ids)
        }).unzip();

        let padded_vertices_length = Self::round_up_to_mul_of_four(vertices.len()) * mem::size_of::<Vertex>();
        let padded_indices_length = indices.iter().map(|v| Self::round_up_to_mul_of_four(v.len()) * mem::size_of::<u32>()).collect::<Vec<_>>();

        let buffer_length = padded_vertices_length + padded_indices_length.iter().sum::<usize>();
        let buffer = root.push(Buffer {
            byte_length: USize64::from(buffer_length),
            name: None,
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let vertices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(padded_vertices_length),
            byte_offset: None,
            byte_stride: Some(Stride(mem::size_of::<Vertex>())),
            name: None,
            target: Some(Valid(ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let indices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(padded_indices_length.iter().sum::<usize>()),
            byte_offset: Some(USize64::from(padded_vertices_length)),
            byte_stride: None,
            name: None,
            target: Some(Valid(ElementArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let (min, max) = {
            let [min_x, min_y, min_z] = vmesh.bounds.0.as_::<f32>().data;
            let [max_x, max_y, max_z] = vmesh.bounds.1.as_::<f32>().data;

            // gltfの座標系に合わせる
            let min = [min_x, min_z, -max_y];
            let max = [max_x, max_z, -min_y];

            (min, max)
        };


        let positions_accessor = root.push(Accessor {
            buffer_view: Some(vertices_buffer_view),
            byte_offset: Some(USize64(0)),
            count: USize64::from(vertices.len()),
            component_type: Valid(GenericComponentType(ComponentType::F32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(Type::Vec3),
            min: Some(Value::from(Vec::from(min))),
            max: Some(Value::from(Vec::from(max))),
            name: None,
            normalized: false,
            sparse: None,
        });

        let primitives = colors.into_iter().enumerate().map(|(i, color)| {
            let offset = padded_indices_length[0..i].iter().sum::<usize>();

            let indices_accessor = root.push(Accessor {
                buffer_view: Some(indices_buffer_view),
                byte_offset: Some(USize64::from(offset)),
                count: USize64::from(indices[i].len()),
                component_type: Valid(GenericComponentType(ComponentType::U32)),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(Type::Scalar),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let pbr_metallic_roughness = PbrMetallicRoughness {
                base_color_factor: PbrBaseColorFactor(color),
                base_color_texture: None,
                metallic_factor: Default::default(),
                roughness_factor: Default::default(),
                metallic_roughness_texture: None,
                extensions: Default::default(),
                extras: Default::default(),
            };

            let material = root.push(Material {
                alpha_cutoff: None,
                alpha_mode: Default::default(),
                double_sided: false,
                name: None,
                pbr_metallic_roughness,
                normal_texture: None,
                occlusion_texture: None,
                emissive_texture: None,
                emissive_factor: Default::default(),
                extensions: Default::default(),
                extras: Default::default(),
            });


            Primitive {
                attributes: BTreeMap::from([(Valid(Semantic::Positions), positions_accessor)]),
                extensions: None,
                extras: Default::default(),
                indices: Some(indices_accessor),
                material: Some(material),
                mode: Valid(Mode::Triangles),
                targets: None,
            }
        }).collect::<Vec<_>>();

        let mesh = root.push(Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            primitives,
            weights: None,
        });

        let node = root.push(Node {
            mesh: Some(mesh),
            translation: Some((vmesh.offset.as_::<f32>() * Point3D::from(-1.)).data),
            // scale: Some([voxel_mesh.resolution as f32; 3]),
            ..Default::default()
        });

        let scene = root.push(Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            nodes: vec![node],
        });

        root.scene = Some(scene);

        let json = root.to_string().map_err(|_| anyhow!("Serialization error"))?.into_bytes();
        let json_offset = Self::round_up_to_mul_of_four(json.len());

        let bin = [
            Self::convert_to_byte_vec(Self::pad_to_mul_of_four(vertices)),
            indices.into_iter().flat_map(|v| Self::convert_to_byte_vec(Self::pad_to_mul_of_four(v))).collect::<Vec<_>>(),
        ].concat();

        Ok(Glb {
            header: Header {
                magic: *b"glTF",
                version: 2,
                length: (json_offset + buffer_length).try_into().map_err(|_| anyhow!("file size exceeds binary glTF limit"))?,
            },
            json: Owned(json),
            bin: Some(Owned(bin)),
        })
    }
}

impl GlbGenPrivateMethod for Glb<'_> {}

impl WaterGlbGen<'_> for Glb<'_> {}
