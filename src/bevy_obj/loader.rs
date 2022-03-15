use anyhow::Result;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::math::Vec3;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use bevy::utils::BoxedFuture;
use thiserror::Error;

#[derive(Default)]
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { Ok(load_obj(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["obj"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum ObjError {
    #[error("Invalid OBJ file: {0}")]
    Gltf(#[from] obj::ObjError),
    #[error("Unknown vertex format")]
    UnknownVertexFormat,
}

async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), ObjError> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    load_obj_from_bytes(bytes, &mut mesh)?;
    load_context.set_default_asset(LoadedAsset::new(mesh));
    Ok(())
}

pub fn load_obj_from_bytes(bytes: &[u8], mesh: &mut Mesh) -> Result<(), ObjError> {
    let raw = obj::raw::parse_obj(bytes)?;

    // Get the most complete vertex representation
    //  1 => Position
    //  2 => Position, Normal
    //  3 => Position, Normal, Texture
    let mut pnt = 4;
    for polygon in &raw.polygons {
        use obj::raw::object::Polygon;
        match polygon {
            Polygon::P(_) => pnt = std::cmp::min(pnt, 1),
            Polygon::PT(_) => pnt = std::cmp::min(pnt, 1),
            Polygon::PN(_) => pnt = std::cmp::min(pnt, 2),
            Polygon::PTN(_) => pnt = std::cmp::min(pnt, 3),
        }
    }

    match pnt {
        1 => {
            let obj: obj::Obj<obj::Position, u32> = obj::Obj::new(raw)?;
            set_position_data(mesh, obj.vertices.iter().map(|v| v.position).collect());
            set_normal_data(mesh, obj.vertices.iter().map(|_| [0., 0., 0.]).collect());
            set_uv_data(mesh, obj.vertices.iter().map(|_| [0., 0.]).collect());
            set_mesh_indices(mesh, obj);
        }
        2 => {
            let obj: obj::Obj<obj::Vertex, u32> = obj::Obj::new(raw)?;
            set_position_data(mesh, obj.vertices.iter().map(|v| v.position).collect());
            set_normal_data(mesh, obj.vertices.iter().map(|v| v.normal).collect());
            set_uv_data(mesh, obj.vertices.iter().map(|_| [0., 0.]).collect());
            set_mesh_indices(mesh, obj);
        }
        3 => {
            let obj: obj::Obj<obj::TexturedVertex, u32> = obj::Obj::new(raw)?;
            set_position_data(mesh, obj.vertices.iter().map(|v| v.position).collect());
            set_normal_data(mesh, obj.vertices.iter().map(|v| v.normal).collect());
            set_uv_data(
                mesh,
                obj.vertices
                    .iter()
                    // Flip UV for correct values
                    .map(|v| [v.texture[0], 1.0 - v.texture[1]])
                    .collect(),
            );
            set_mesh_indices(mesh, obj);
        }
        _ => return Err(ObjError::UnknownVertexFormat),
    }

    Ok(())
}

fn set_position_data(mesh: &mut Mesh, data: Vec<[f32; 3]>) {
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, data);
}

fn set_normal_data(mesh: &mut Mesh, data: Vec<[f32; 3]>) {
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, data);
}

fn set_uv_data(mesh: &mut Mesh, data: Vec<[f32; 2]>) {
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, data);
}

fn set_mesh_indices<T>(mesh: &mut Mesh, obj: obj::Obj<T, u32>) {
    // Invert faces
    let mut indicies: Vec<_> = obj.indices.iter().map(|i| *i as u32).collect();
    let pos = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => v,
        _ => panic!("This should never happen"),
    };
    let normal = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(v)) => v,
        _ => panic!("This should never happen"),
    };
    for i in 0..indicies.len()/3 {
        let (a, b, c) = (indicies[i*3] as usize, indicies[i*3+1] as usize, indicies[i*3+2] as usize);
        let ab = Vec3::new(pos[a][0] - pos[b][0], pos[a][1] - pos[b][1], pos[a][2] - pos[b][2]);
        let ac = Vec3::new(pos[a][0] - pos[c][0], pos[a][1] - pos[c][1], pos[a][2] - pos[c][2]);
        let dot = ab.cross(ac).dot(Vec3::new(normal[a][0], normal[a][1], normal[a][2]));
        if dot < 0. {
            indicies.swap(i * 3 + 1, i * 3 + 2);
        }
    }
    mesh.set_indices(Some(Indices::U32(indicies)));
}
