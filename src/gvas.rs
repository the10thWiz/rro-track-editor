// Format: 'GVAS'
// SaveGameVersion = reader.ReadInt32();
// PackageVersion = reader.ReadInt32();
// EngineVersion.Major = reader.ReadInt16();
// EngineVersion.Minor = reader.ReadInt16();
// EngineVersion.Patch = reader.ReadInt16();
// EngineVersion.Build = reader.ReadInt32();
// EngineVersion.BuildId = reader.ReadUEString();
// CustomFormatVersion = reader.ReadInt32();
// CustomFormatData.Count = reader.ReadInt32();

use std::{
    io::{Error, ErrorKind, Read, Result, Write},
    mem::size_of,
    sync::atomic::{AtomicBool, Ordering},
};

pub trait ReadExt: Read {
    fn read_string(&mut self) -> Result<String>;
    fn read_string_len(&mut self, len: i64) -> Result<String>;
    fn read_u64(&mut self) -> Result<u64>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_guid(&mut self) -> Result<()>;
}
trait WriteExt: Write {
    fn write_string(&mut self, s: &str) -> Result<()> {
        self.write_all(&(s.len() as u32 + 1).to_le_bytes())?;
        self.write_all(s.as_bytes())?;
        self.write_all(&[0u8])
    }
}

impl<W: Write> WriteExt for W {}

impl<R: Read> ReadExt for R {
    fn read_string(&mut self) -> Result<String> {
        let len = self.read_i32()?;
        //println!("String len: {:X}", len);
        if len > 0 {
            let mut buf = vec![0u8; len as usize];
            self.read_exact(&mut buf)?;
            let null_byte = buf.pop().unwrap();
            //if null_byte != 0 {
            //return Err(Error::new(
            //std::io::ErrorKind::InvalidData,
            //"String not terminated",
            //));
            //}
            Ok(encoding_rs::WINDOWS_1252
                .decode_without_bom_handling(&buf)
                .0
                .into_owned())
        } else if len < 0 {
            let mut buf = vec![0u8; len.abs() as usize * 2];
            self.read_exact(&mut buf)?;
            let (e, e2) = (buf.pop(), buf.pop());
            //if e != Some(0) || e2 != Some(0) {
            //return Err(Error::new(
            //std::io::ErrorKind::InvalidData,
            //"String not terminated",
            //));
            //}
            Ok(encoding_rs::UTF_16LE
                .decode_without_bom_handling(&buf)
                .0
                .into_owned())
        } else {
            Ok(String::new())
        }
    }

    fn read_string_len(&mut self, exp_len: i64) -> Result<String> {
        let len = self.read_i32()?;
        assert_eq!(len as usize + size_of::<i32>(), exp_len as usize);
        //println!("String len: {:X}", len);
        if len > 0 {
            let mut buf = vec![0u8; len as usize];
            self.read_exact(&mut buf)?;
            let null_byte = buf.pop().unwrap();
            //if null_byte != 0 {
            //return Err(Error::new(
            //std::io::ErrorKind::InvalidData,
            //"String not terminated",
            //));
            //}
            Ok(encoding_rs::WINDOWS_1252
                .decode_without_bom_handling(&buf)
                .0
                .into_owned())
        } else if len < 0 {
            let mut buf = vec![0u8; len.abs() as usize * 2];
            self.read_exact(&mut buf)?;
            let (e, e2) = (buf.pop(), buf.pop());
            //if e != Some(0) || e2 != Some(0) {
            //return Err(Error::new(
            //std::io::ErrorKind::InvalidData,
            //"String not terminated",
            //));
            //}
            Ok(encoding_rs::UTF_16LE
                .decode_without_bom_handling(&buf)
                .0
                .into_owned())
        } else {
            Ok(String::new())
        }
    }

    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; size_of::<f32>()];
        self.read_exact(&mut buf)?;
        Ok(f32::from_ne_bytes(buf))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; size_of::<u64>()];
        self.read_exact(&mut buf)?;
        Ok(u64::from_ne_bytes(buf))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let mut buf = [0u8; size_of::<i64>()];
        self.read_exact(&mut buf)?;
        Ok(i64::from_ne_bytes(buf))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; size_of::<u32>()];
        self.read_exact(&mut buf)?;
        Ok(u32::from_ne_bytes(buf))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; size_of::<i32>()];
        self.read_exact(&mut buf)?;
        Ok(i32::from_ne_bytes(buf))
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; size_of::<u16>()];
        self.read_exact(&mut buf)?;
        Ok(u16::from_ne_bytes(buf))
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; size_of::<u8>()];
        self.read_exact(&mut buf)?;
        Ok(u8::from_ne_bytes(buf))
    }

    fn read_guid(&mut self) -> Result<()> {
        let mut buf = [0u8; 16];
        self.read_exact(&mut buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GVASFile {
    save_game_version: u32,
    package_version: u32,
    engine_version: EngineVersion,
    custom_format_version: u32,
    // format_data_count: u32,
    custom_format_data: Vec<DataEntry>,
    save_game_type: String,
    properties: Vec<Property>,
}

impl GVASFile {
    pub fn read(r: &mut impl ReadExt) -> Result<Self> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        assert_eq!(&buf, b"GVAS", "Unexpected Header");
        //r.read_exact(&mut buf)?;
        let save_game_version = r.read_u32()?;
        //dbg!(save_game_version);
        let package_version = r.read_u32()?;
        //dbg!(package_version);
        let engine_version = EngineVersion::read(r)?;
        //dbg!(&engine_version);
        let custom_format_version = r.read_u32()?;
        //dbg!(custom_format_version);
        let custom_format_count = r.read_u32()?;
        //dbg!(custom_format_count);
        let custom_format_data = (0..custom_format_count)
            .map(|_| DataEntry::read(r))
            .collect::<Result<_>>()?;
        //dbg!(&custom_format_data);
        let save_game_type = r.read_string()?;
        //dbg!(&save_game_type);
        let mut properties = vec![];
        while let Some(prop) = Property::read(r)? {
            //dbg!(&prop);
            properties.push(prop);
        }
        let mut buf = [0u8; 100];
        let len = r.read(&mut buf)?;
        //println!("rem: {:?}\n", &buf[..len]);
        Ok(Self {
            save_game_version,
            package_version,
            engine_version,
            custom_format_version,
            custom_format_data,
            save_game_type,
            properties,
        })
    }

    pub fn write(self, w: &mut impl Write) -> Result<()> {
        write!(w, "GVAS")?;
        w.write_all(&self.save_game_version.to_le_bytes())?;
        w.write_all(&self.package_version.to_le_bytes())?;
        self.engine_version.write(w)?;
        w.write_all(&self.custom_format_version.to_le_bytes())?;
        w.write_all(&(self.custom_format_data.len() as u32).to_le_bytes())?;
        for entry in self.custom_format_data {
            entry.write(w)?;
        }
        w.write_string(self.save_game_type.as_str())?;
        for prop in self.properties {
            prop.write(w)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct EngineVersion {
    major: u16,
    minor: u16,
    patch: u16,
    build: u32,
    build_id: String,
}

impl EngineVersion {
    pub fn read(r: &mut impl ReadExt) -> Result<Self> {
        let major = r.read_u16()?;
        //dbg!(major);
        let minor = r.read_u16()?;
        //dbg!(minor);
        let patch = r.read_u16()?;
        //dbg!(patch);
        let build = r.read_u32()?;
        //dbg!(build);
        let build_id = r.read_string()?;
        //dbg!(&build_id);
        Ok(Self {
            major,
            minor,
            patch,
            build,
            build_id,
        })
    }

    pub fn write(self, w: &mut impl Write) -> Result<()> {
        w.write_all(&self.major.to_le_bytes())?;
        w.write_all(&self.minor.to_le_bytes())?;
        w.write_all(&self.patch.to_le_bytes())?;
        w.write_all(&self.build.to_le_bytes())?;
        w.write_string(self.build_id.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DataEntry {
    guid: [u8; 16],
    value: u32,
}

impl DataEntry {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let mut guid = [0u8; 16];
        r.read_exact(&mut guid)?;
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        let value = u32::from_ne_bytes(buf);
        Ok(Self { guid, value })
    }

    pub fn write(self, w: &mut impl Write) -> Result<()> {
        w.write_all(&self.guid)?;
        w.write_all(&self.value.to_le_bytes())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Property {
    name: String,
    val: Value,
}

impl Property {
    pub fn read(r: &mut impl Read) -> Result<Option<Self>> {
        let name = match r.read_string() {
            Ok(name) => name,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        };
        //dbg!(&name);
        let val = Value::read(r)?;
        //dbg!(&val);
        Ok(Some(Self { name, val }))
    }

    pub fn write(self, w: &mut impl Write) -> Result<()> {
        w.write_string(self.name.as_str())?;
        self.val.write(w)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Bool(bool),
    Int32(u32),
    //Float(f32),
    //Name(String),
    //String(String),
    //Text(Vec<String>),
    //Enum(),
    //StructProperty(String, Box<Self>),
    Array(String, usize, Option<(String, u64)>, Vec<Self>),
    //Map(),
    //Byte(),
    Vector([f32; 3]),
    Raw(String, Vec<u8>),
    RawArray(String, usize, Vec<u8>),
    None,
}

impl Value {
    pub fn read(r: &mut impl ReadExt) -> Result<Self> {
        match Self::read_in(r) {
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => Ok(Self::None),
            o => o,
        }
    }

    pub fn read_in(r: &mut impl ReadExt) -> Result<Self> {
        let ty = r.read_string()?;
        //dbg!(&ty);
        let len = r.read_i64()?;
        //dbg!(len);
        Self::read_val(r, ty, len)
    }

    pub fn read_val(r: &mut impl ReadExt, ty: String, len: i64) -> Result<Self> {
        Ok(match ty.as_str() {
            "ArrayProperty" => {
                //r.read_u8()?;
                let arr_ty = r.read_string()?;
                //dbg!(&arr_ty);
                r.read_u8()?;
                let arr_len = r.read_u32()?;
                //dbg!(arr_len);
                //let mut ret = Vec::with_capacity(arr_len as usize);
                let left = len - 4;
                //let unit = left / arr_len as i64;
                //dbg!(unit);
                //assert_eq!(unit * arr_len as i64, left);
                //for _ in 0..left {

                ////ret.push(Self::read_val(r, arr_ty.clone(), left)?);
                //}
                let mut buf = vec![0u8; left as usize];
                r.read_exact(&mut buf)?;
                //println!("{:X?}", buf);
                //panic!();
                Self::RawArray(arr_ty, arr_len as usize, buf)
            }
            _ => {
                let mut buf = vec![0u8; len as usize + 1];
                r.read_exact(&mut buf)?;
                Self::Raw(ty, buf)
            }
        })
    }

    pub fn translate(&mut self) -> Result<()> {
        *self = match self {
            Self::RawArray(ty, len, raw) => {
                Self::read_array(ty.as_str(), *len, std::mem::take(raw))?
            }
            _ => return Ok(()),
        };
        Ok(())
    }

    fn read_array(ty: &str, len: usize, raw: Vec<u8>) -> Result<Self> {
        //dbg!(&ty);
        let raw_len = raw.len();
        let mut raw = std::io::Cursor::new(raw);
        match ty {
            "StructProperty" => {
                let name = raw.read_string()?;
                //dbg!(&name);
                let ty = raw.read_string()?;
                //dbg!(&ty);
                let inner_len = raw.read_u64()?;
                //dbg!(_pad);
                let ty = raw.read_string()?;
                //dbg!(&ty);
                match ty.as_str() {
                    "Vector" => {
                        raw.read_guid()?;
                        let _pad = raw.read_u8()?;
                        let mut ret = Vec::with_capacity(len);
                        for _ in 0..len {
                            let x = raw.read_f32()?;
                            let y = raw.read_f32()?;
                            let z = raw.read_f32()?;
                            ret.push(Self::Vector([x, y, z]));
                        }
                        //dbg!(&ret);
                        //println!("{:X?}", raw.bytes().collect::<Result<Vec<_>>>()?);
                        Ok(Self::Array(
                            ty.to_string(),
                            raw_len,
                            Some((name, inner_len)),
                            ret,
                        ))
                    }
                    _ => todo!(),
                }
            }
            "IntProperty" => {
                let mut ret = Vec::with_capacity(len);
                for _ in 0..len {
                    let x = raw.read_u32()?;
                    ret.push(Self::Int32(x));
                }
                //dbg!(&ret);
                //println!("{:X?}", raw.bytes().collect::<Result<Vec<_>>>()?);
                Ok(Self::Array(ty.to_string(), raw_len, None, ret))
            }
            "BoolProperty" => {
                let mut ret = Vec::with_capacity(len);
                for _ in 0..len {
                    let x = raw.read_u8()?;
                    ret.push(Self::Bool(x != 0));
                }
                //dbg!(&ret);
                //println!("{:X?}", raw.bytes().collect::<Result<Vec<_>>>()?);
                Ok(Self::Array(ty.to_string(), raw_len, None, ret))
            }
            _ => todo!(),
        }
    }

    pub fn write(self, w: &mut impl Write) -> Result<()> {
        match self {
            Self::Raw(ty, raw) => {
                w.write_string(ty.as_str())?;
                w.write_all(&(raw.len() as i64 - 1).to_le_bytes())?;
                w.write_all(raw.as_slice())
            }
            Self::RawArray(ty, len, raw) => {
                w.write_string("ArrayProperty")?;
                w.write_all(&(raw.len() as i64 + 4).to_le_bytes())?;
                w.write_string(ty.as_str())?;
                w.write_all(&[0])?;
                w.write_all(&(len as u32).to_le_bytes())?;
                w.write_all(raw.as_slice())
            }
            Self::None => w.write_all(&[0, 0, 0, 0]),
            Self::Array(ty, raw_len, vecdata, inner) => {
                w.write_string("ArrayProperty")?;
                match ty.as_str() {
                    "Vector" => {
                        let (name, inner_len) = vecdata.unwrap();
                        let raw_len = raw_len - inner_len as usize;
                        let inner_len = inner.len() * size_of::<f32>() * 3;
                        let raw_len = raw_len + inner_len;
                        w.write_all(&(raw_len as i64 + 4).to_le_bytes())?;
                        w.write_string("StructProperty")?;
                        w.write_all(&[0])?;
                        w.write_all(&(inner.len() as u32).to_le_bytes())?;

                        w.write_string(name.as_str())?;
                        w.write_string("StructProperty")?;
                        w.write_all(&(inner_len as u64).to_le_bytes())?;
                        w.write_string(ty.as_str())?;
                        w.write_all(&[0u8; 17])?;
                    }
                    _ => {
                        let raw_len = match ty.as_str() {
                            "BoolProperty" => inner.len(),
                            "IntProperty" => inner.len() * size_of::<u32>(),
                            _ => todo!(),
                        };
                        w.write_all(&(raw_len as i64 + 4).to_le_bytes())?;
                        w.write_string(ty.as_str())?;
                        w.write_all(&[0])?;
                        w.write_all(&(inner.len() as u32).to_le_bytes())?;
                    }
                }
                for v in inner {
                    v.write(w)?;
                }
                Ok(())
            }
            Self::Vector(v) => {
                w.write_all(&v[0].to_le_bytes())?;
                w.write_all(&v[1].to_le_bytes())?;
                w.write_all(&v[2].to_le_bytes())
            }
            Self::Int32(v) => w.write_all(&v.to_le_bytes()),
            Self::Bool(v) => w.write_all(&[if v { 1u8 } else { 0u8 }]),
            //_ => todo!("{:?}", self),
        }
    }

    fn set_array(&mut self, v: Vec<Self>) {
        match self {
            Self::Array(_, _, _, a) => *a = v,
            _ => panic!("Not array"),
        }
    }
}

pub struct RROSave {
    inner: GVASFile,
    pub spline_location_array: Vec<[f32; 3]>,
    pub spline_type_array: Vec<u32>,
    pub spline_control_points_array: Vec<[f32; 3]>,
    pub spline_control_points_index_start_array: Vec<u32>,
    pub spline_control_points_index_end_array: Vec<u32>,
    pub spline_segments_visibility_array: Vec<bool>,
    pub spline_visibility_start_array: Vec<u32>,
    pub spline_visibility_end_array: Vec<u32>,
    idx_spline_location_array: usize,
    idx_spline_type_array: usize,
    idx_spline_control_points_array: usize,
    idx_spline_control_points_index_start_array: usize,
    idx_spline_control_points_index_end_array: usize,
    idx_spline_segments_visibility_array: usize,
    idx_spline_visibility_start_array: usize,
    idx_spline_visibility_end_array: usize,
}

impl RROSave {
    pub fn read(r: &mut impl ReadExt) -> Result<Self> {
        Ok(Self::from_gvas(GVASFile::read(r)?))
    }

    pub fn from_gvas(mut inner: GVASFile) -> Self {
        let mut spline_location_array = vec![];
        let mut spline_type_array = vec![];
        let mut spline_control_points_array = vec![];
        let mut spline_control_points_index_start_array = vec![];
        let mut spline_control_points_index_end_array = vec![];
        let mut spline_segments_visibility_array = vec![];
        let mut spline_visibility_start_array = vec![];
        let mut spline_visibility_end_array = vec![];
        let mut idx_spline_location_array = 0;
        let mut idx_spline_type_array = 0;
        let mut idx_spline_control_points_array = 0;
        let mut idx_spline_control_points_index_start_array = 0;
        let mut idx_spline_control_points_index_end_array = 0;
        let mut idx_spline_segments_visibility_array = 0;
        let mut idx_spline_visibility_start_array = 0;
        let mut idx_spline_visibility_end_array = 0;
        for (i, prop) in inner.properties.iter_mut().enumerate() {
            prop.val.translate().unwrap();
            if let Value::Array(_, _, _, v) = &prop.val {
                match prop.name.as_str() {
                    "SplineLocationArray" => {
                        spline_location_array = v
                            .iter()
                            .map(|v| {
                                if let Value::Vector(v) = v {
                                    *v
                                } else {
                                    panic!()
                                }
                            })
                            .collect();
                        idx_spline_location_array = i;
                    }
                    "SplineTypeArray" => {
                        spline_type_array = v
                            .iter()
                            .map(|v| {
                                if let Value::Int32(v) = v {
                                    *v as u32
                                } else {
                                    panic!()
                                }
                            })
                            .collect();
                        idx_spline_type_array = i;
                    }
                    "SplineControlPointsArray" => {
                        spline_control_points_array = v
                            .iter()
                            .map(|v| {
                                if let Value::Vector(v) = v {
                                    *v
                                } else {
                                    panic!()
                                }
                            })
                            .collect();
                        idx_spline_control_points_array = i;
                    }
                    "SplineControlPointsIndexStartArray" => {
                        spline_control_points_index_start_array = v
                            .iter()
                            .map(|v| {
                                if let Value::Int32(v) = v {
                                    *v as u32
                                } else {
                                    panic!()
                                }
                            })
                            .collect();
                        idx_spline_control_points_index_start_array = i;
                    }
                    "SplineControlPointsIndexEndArray" => {
                        spline_control_points_index_end_array = v
                            .iter()
                            .map(|v| {
                                if let Value::Int32(v) = v {
                                    *v as u32
                                } else {
                                    panic!()
                                }
                            })
                            .collect();
                        idx_spline_control_points_index_end_array = i;
                    }
                    "SplineSegmentsVisibilityArray" => {
                        spline_segments_visibility_array = v
                            .iter()
                            .map(|v| if let Value::Bool(v) = v { *v } else { panic!() })
                            .collect();
                        idx_spline_segments_visibility_array = i;
                    }
                    "SplineVisibilityStartArray" => {
                        spline_visibility_start_array = v
                            .iter()
                            .map(|v| {
                                if let Value::Int32(v) = v {
                                    *v as u32
                                } else {
                                    panic!()
                                }
                            })
                            .collect();
                        idx_spline_visibility_start_array = i;
                    }
                    "SplineVisibilityEndArray" => {
                        spline_visibility_end_array = v
                            .iter()
                            .map(|v| {
                                if let Value::Int32(v) = v {
                                    *v as u32
                                } else {
                                    panic!()
                                }
                            })
                            .collect();
                        idx_spline_visibility_end_array = i;
                    }
                    _ => (),
                }
            }
        }
        Self {
            inner,
            spline_location_array,
            spline_type_array,
            spline_control_points_array,
            spline_control_points_index_start_array,
            spline_control_points_index_end_array,
            spline_segments_visibility_array,
            spline_visibility_start_array,
            spline_visibility_end_array,
            idx_spline_location_array,
            idx_spline_type_array,
            idx_spline_control_points_array,
            idx_spline_control_points_index_start_array,
            idx_spline_control_points_index_end_array,
            idx_spline_segments_visibility_array,
            idx_spline_visibility_start_array,
            idx_spline_visibility_end_array,
        }
    }

    pub fn write(mut self, w: &mut impl Write) -> Result<()> {
        self.inner.properties[self.idx_spline_location_array]
            .val
            .set_array(
                self.spline_location_array
                    .into_iter()
                    .map(|f| Value::Vector(f))
                    .collect(),
            );
        self.inner.properties[self.idx_spline_type_array]
            .val
            .set_array(
                self.spline_type_array
                    .into_iter()
                    .map(|f| Value::Int32(f))
                    .collect(),
            );
        self.inner.properties[self.idx_spline_control_points_array]
            .val
            .set_array(
                self.spline_control_points_array
                    .into_iter()
                    .map(|f| Value::Vector(f))
                    .collect(),
            );
        self.inner.properties[self.idx_spline_control_points_index_start_array]
            .val
            .set_array(
                self.spline_control_points_index_start_array
                    .into_iter()
                    .map(|f| Value::Int32(f))
                    .collect(),
            );
        self.inner.properties[self.idx_spline_control_points_index_end_array]
            .val
            .set_array(
                self.spline_control_points_index_end_array
                    .into_iter()
                    .map(|f| Value::Int32(f))
                    .collect(),
            );
        self.inner.properties[self.idx_spline_segments_visibility_array]
            .val
            .set_array(
                self.spline_segments_visibility_array
                    .into_iter()
                    .map(|f| Value::Bool(f))
                    .collect(),
            );
        self.inner.properties[self.idx_spline_visibility_start_array]
            .val
            .set_array(
                self.spline_visibility_start_array
                    .into_iter()
                    .map(|f| Value::Int32(f))
                    .collect(),
            );
        self.inner.properties[self.idx_spline_visibility_end_array]
            .val
            .set_array(
                self.spline_visibility_end_array
                    .into_iter()
                    .map(|f| Value::Int32(f))
                    .collect(),
            );
        self.inner.write(w)
    }
}

fn main() {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut ret = GVASFile::read(&mut stdin).expect("io error");
    for prop in ret.properties.iter_mut().skip(8).take(8) {
        prop.val.translate().unwrap();
        eprintln!(": {}: {:X?}", prop.name, prop.val);
    }
    ret.write(&mut stdout).expect("write error");
    //if let Value::Array(tmp) = &ret.properties.last().unwrap().val {
    //for prop in tmp {
    //println!("   : {:?}", prop);
    //}
    //}

    //println!("{:?}", ret.properties);
}
