use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Seek, SeekFrom, Write},
    mem::size_of,
};

#[derive(Debug)]
pub enum GVASError {
    IOError(Error),
    Missing(&'static str),
    WrongType,
}

impl From<Error> for GVASError {
    fn from(e: Error) -> Self {
        Self::IOError(e)
    }
}

pub type Result<T> = std::result::Result<T, GVASError>;

pub trait ReadExt: Read {
    fn read_uestring(&mut self) -> Result<String>;
    fn read_string_len(&mut self, len: i64) -> Result<String>;
    fn read_u64(&mut self) -> Result<u64>;
    fn read_i64(&mut self) -> Result<i64>;
    fn read_u32(&mut self) -> Result<u32>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_i8(&mut self) -> Result<i8>;
    fn read_guid(&mut self) -> Result<()>;
}
trait WriteExt: Write {
    fn write_string(&mut self, s: &str) -> Result<()> {
        if s != "" {
            self.write_all(&(s.len() as u32 + 1).to_le_bytes())?;
            self.write_all(s.as_bytes())?;
            self.write_all(&[0u8])?;
        } else {
            self.write_all(&0u32.to_le_bytes())?;
        }
        Ok(())
    }
}

impl<W: Write> WriteExt for W {}

impl<R: Read> ReadExt for R {
    fn read_uestring(&mut self) -> Result<String> {
        let len = self.read_i32()?;
        if len > 0 {
            let mut buf = vec![0u8; len as usize];
            self.read_exact(&mut buf)?;
            let null_byte = buf.pop().unwrap();
            if null_byte != 0 {
                return Err(
                    Error::new(std::io::ErrorKind::InvalidData, "String not terminated").into(),
                );
            }
            Ok(encoding_rs::WINDOWS_1252
                .decode_without_bom_handling(&buf)
                .0
                .into_owned())
        } else if len < 0 {
            let mut buf = vec![0u8; len.abs() as usize * 2];
            self.read_exact(&mut buf)?;
            let (e, e2) = (buf.pop(), buf.pop());
            if e != Some(0) || e2 != Some(0) {
                return Err(
                    Error::new(std::io::ErrorKind::InvalidData, "String not terminated").into(),
                );
            }
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
        if len > 0 {
            let mut buf = vec![0u8; len as usize];
            self.read_exact(&mut buf)?;
            let null_byte = buf.pop().unwrap();
            if null_byte != 0 {
                return Err(
                    Error::new(std::io::ErrorKind::InvalidData, "String not terminated").into(),
                );
            }
            Ok(encoding_rs::WINDOWS_1252
                .decode_without_bom_handling(&buf)
                .0
                .into_owned())
        } else if len < 0 {
            let mut buf = vec![0u8; len.abs() as usize * 2];
            self.read_exact(&mut buf)?;
            let (e, e2) = (buf.pop(), buf.pop());
            if e != Some(0) || e2 != Some(0) {
                return Err(
                    Error::new(std::io::ErrorKind::InvalidData, "String not terminated").into(),
                );
            }
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

    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0u8; size_of::<i8>()];
        self.read_exact(&mut buf)?;
        Ok(i8::from_ne_bytes(buf))
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
        let save_game_version = r.read_u32()?;
        let package_version = r.read_u32()?;
        let engine_version = EngineVersion::read(r)?;
        let custom_format_version = r.read_u32()?;
        let custom_format_count = r.read_u32()?;
        let custom_format_data = (0..custom_format_count)
            .map(|_| DataEntry::read(r))
            .collect::<Result<_>>()?;
        let save_game_type = r.read_uestring()?;
        let mut properties = vec![];
        while let Some(prop) = Property::read(r)? {
            properties.push(prop);
        }
        let mut buf = [0u8; 100];
        let _len = r.read(&mut buf)?;
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

    pub fn write(&self, w: &mut (impl Write + Seek)) -> Result<()> {
        write!(w, "GVAS")?;
        w.write_all(&self.save_game_version.to_le_bytes())?;
        w.write_all(&self.package_version.to_le_bytes())?;
        self.engine_version.write(w)?;
        w.write_all(&self.custom_format_version.to_le_bytes())?;
        w.write_all(&(self.custom_format_data.len() as u32).to_le_bytes())?;
        for entry in &self.custom_format_data {
            entry.write(w)?;
        }
        w.write_string(self.save_game_type.as_str())?;
        for prop in &self.properties {
            prop.write(w)?;
        }
        Ok(())
    }

    fn get_prop<'a>(&'a self, name: &'static str) -> Result<&'a Value> {
        self.properties
            .iter()
            .find(|p| p.name == name)
            .map(|p| &p.val)
            .ok_or_else(|| GVASError::Missing(name))
    }

    fn get_prop_mut<'a>(&'a mut self, name: &'static str) -> Result<&'a mut Value> {
        self.properties
            .iter_mut()
            .find(|p| p.name == name)
            .map(|p| &mut p.val)
            .ok_or_else(|| GVASError::Missing(name))
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
        let minor = r.read_u16()?;
        let patch = r.read_u16()?;
        let build = r.read_u32()?;
        let build_id = r.read_uestring()?;
        Ok(Self {
            major,
            minor,
            patch,
            build,
            build_id,
        })
    }

    pub fn write(&self, w: &mut impl Write) -> Result<()> {
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

    pub fn write(&self, w: &mut (impl Write + Seek)) -> Result<()> {
        w.write_all(&self.guid)?;
        w.write_all(&self.value.to_le_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Property {
    name: String,
    val: Value,
}

impl Property {
    pub fn read(r: &mut impl Read) -> Result<Option<Self>> {
        let name = match r.read_uestring() {
            Ok(name) => name,
            Err(GVASError::IOError(e)) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        };
        let val = Value::read(r, name.as_str())?;
        Ok(Some(Self { name, val }))
    }

    pub fn write(&self, w: &mut (impl Write + Seek)) -> Result<()> {
        w.write_string(self.name.as_str())?;
        self.val.write(w, self.name.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    StringArray(Vec<String>),
    Int32Array(Vec<u32>),
    BoolArray(Vec<bool>),
    FloatArray(Vec<f32>),
    TextArray(Vec<TextProperty>),
    VectorArray(Vec<[f32; 3]>),
    RotatorArray(Vec<[f32; 3]>),
    None,
}

impl Value {
    pub fn is_array(&self) -> bool {
        match self {
            Self::None | Self::String(_) => false,
            Self::StringArray(_)
            | Self::Int32Array(_)
            | Self::BoolArray(_)
            | Self::FloatArray(_)
            | Self::TextArray(_)
            | Self::VectorArray(_)
            | Self::RotatorArray(_) => true,
        }
    }
    pub fn write(&self, w: &mut (impl Write + Seek), name: &str) -> Result<()> {
        let start = if self.is_array() {
            w.write_string("ArrayProperty")?;
            let start = w.stream_position()?;
            w.write_all(&0u64.to_le_bytes())?;
            Some(start)
        } else {
            None
        };
        let len = match self {
            Self::None => {
                w.write_all(&[0u8; size_of::<u32>()])?;
                0
            }
            Self::String(s) => {
                w.write_string("StrProperty")?;
                let sz = s.len() as u64 + 4 + 1;
                w.write_all(&sz.to_le_bytes())?;
                w.write_all(&0u8.to_le_bytes())?;
                w.write_string(s.as_str())?;
                0
            }
            Self::StringArray(arr) => Self::write_str_array(w, arr)?,
            Self::Int32Array(arr) => Self::write_int_array(w, arr)?,
            Self::FloatArray(arr) => Self::write_float_array(w, arr)?,
            Self::BoolArray(arr) => Self::write_bool_array(w, arr)?,
            Self::VectorArray(arr) => Self::write_struct_array(w, arr, name, "Vector")?,
            Self::RotatorArray(arr) => Self::write_struct_array(w, arr, name, "Rotator")?,
            Self::TextArray(arr) => Self::write_text_array(w, arr)?,
        };
        if let Some(start) = start {
            let end = w.stream_position()?;
            w.seek(SeekFrom::Start(start))?;
            w.write_all(&len.to_le_bytes())?;
            w.seek(SeekFrom::Start(end))?;
        }
        Ok(())
    }

    pub fn write_bool_array(w: &mut impl Write, arr: &Vec<bool>) -> Result<u64> {
        w.write_string("BoolProperty")?;
        w.write_all(&0u8.to_le_bytes())?;
        w.write_all(&(arr.len() as u32).to_le_bytes())?;
        let len = arr.len() as u64 + 4;
        for s in arr {
            w.write_all(&[if *s { 1u8 } else { 0u8 }])?;
        }
        Ok(len)
    }

    pub fn write_float_array(w: &mut impl Write, arr: &Vec<f32>) -> Result<u64> {
        w.write_string("FloatProperty")?;
        w.write_all(&0u8.to_le_bytes())?;
        w.write_all(&(arr.len() as u32).to_le_bytes())?;
        let len = (arr.len() * size_of::<f32>()) as u64 + 4;
        for s in arr {
            w.write_all(&s.to_le_bytes())?;
        }
        Ok(len)
    }

    pub fn write_int_array(w: &mut impl Write, arr: &Vec<u32>) -> Result<u64> {
        w.write_string("IntProperty")?;
        w.write_all(&0u8.to_le_bytes())?;
        w.write_all(&(arr.len() as u32).to_le_bytes())?;
        let len = (arr.len() * size_of::<u32>()) as u64 + 4;
        for s in arr {
            w.write_all(&s.to_le_bytes())?;
        }
        Ok(len)
    }

    pub fn write_str_array(w: &mut impl Write, arr: &Vec<String>) -> Result<u64> {
        w.write_string("StrProperty")?;
        w.write_all(&0u8.to_le_bytes())?;
        w.write_all(&(arr.len() as u32).to_le_bytes())?;
        let mut len = 4;
        for s in arr {
            w.write_string(s.as_str())?;
            len += if s != "" { 5 } else { 4 };
            len += s.len() as u64;
        }
        Ok(len)
    }

    pub fn write_text_array(w: &mut impl Write, arr: &Vec<TextProperty>) -> Result<u64> {
        w.write_string("TextProperty")?;
        w.write_all(&0u8.to_le_bytes())?;
        w.write_all(&(arr.len() as u32).to_le_bytes())?;
        let mut len = 4;
        for t in arr {
            len += t.write(w)?;
        }
        Ok(len)
    }

    pub fn write_struct_array(
        w: &mut impl Write,
        arr: &Vec<[f32; 3]>,
        name: &str,
        ty: &str,
    ) -> Result<u64> {
        w.write_string("StructProperty")?;
        w.write_all(&0u8.to_le_bytes())?;
        let num_el = arr.len() as u32;
        w.write_all(&num_el.to_le_bytes())?;
        let len = 4;

        w.write_string(name)?;
        let len = len + name.len() as u64 + 4 + 1;
        w.write_string("StructProperty")?;
        let len = len + "StructProperty".len() as u64 + 4 + 1;
        w.write_all(&(num_el as u64 * 12).to_le_bytes())?;
        let len = len + 8;

        w.write_string(ty)?;
        let len = len + ty.len() as u64 + 4 + 1;
        w.write_all(&[0u8; 17])?;
        let len = len + 17;
        let len = len + arr.len() as u64 * 12;
        for [a, b, c] in arr {
            w.write_all(&a.to_le_bytes())?;
            w.write_all(&b.to_le_bytes())?;
            w.write_all(&c.to_le_bytes())?;
        }
        Ok(len)
    }

    pub fn read(r: &mut impl Read, name: &str) -> Result<Self> {
        let ty = r.read_uestring()?;
        match ty.as_str() {
            "StrProperty" => Self::read_str(r),
            "ArrayProperty" => Self::read_array(r, name),
            "" => Ok(Self::None),
            _ => todo!("support for {}", ty),
        }
    }

    pub fn read_str(r: &mut impl Read) -> Result<Self> {
        let _sz = r.read_u64()?;
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            Err(Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into())
        } else {
            Ok(Self::String(r.read_uestring()?))
        }
    }

    pub fn read_array(r: &mut impl Read, name: &str) -> Result<Self> {
        let plen = r.read_u64()?;
        let dtype = r.read_uestring()?;
        match dtype.as_str() {
            "StructProperty" => Self::read_struct_array(r, plen, name),
            "BoolProperty" => Self::read_bool_array(r, plen),
            "IntProperty" => Self::read_int_array(r, plen),
            "FloatProperty" => Self::read_float_array(r, plen),
            "StrProperty" => Self::read_str_array(r, plen),
            "TextProperty" => Self::read_text_array(r, plen),
            a => return Err(Error::new(ErrorKind::InvalidData, format!("Unimplemented array type: {}", a)).into()),
        }
    }

    pub fn read_bool_array(r: &mut impl Read, _plen: u64) -> Result<Self> {
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            return Err(
                Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into(),
            );
        }
        let nint = r.read_u32()?;
        let mut data = Vec::with_capacity(nint as usize);
        for _ in 0..nint {
            data.push(r.read_u8()? != 0);
        }
        Ok(Self::BoolArray(data))
    }

    pub fn read_float_array(r: &mut impl Read, _plen: u64) -> Result<Self> {
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            return Err(
                Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into(),
            );
        }
        let nint = r.read_u32()?;
        let mut data = Vec::with_capacity(nint as usize);
        for _ in 0..nint {
            data.push(r.read_f32()?);
        }
        Ok(Self::FloatArray(data))
    }

    pub fn read_int_array(r: &mut impl Read, _plen: u64) -> Result<Self> {
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            return Err(
                Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into(),
            );
        }
        let nint = r.read_u32()?;
        let mut data = Vec::with_capacity(nint as usize);
        for _ in 0..nint {
            data.push(r.read_u32()?);
        }
        Ok(Self::Int32Array(data))
    }

    pub fn read_struct_array(r: &mut impl Read, _plen: u64, name: &str) -> Result<Self> {
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            return Err(
                Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into(),
            );
        }
        let struct_size = r.read_u32()?;
        let pname = r.read_uestring()?;
        assert_eq!(pname, name, "Struct Array Name");
        assert_eq!(
            r.read_uestring()?,
            "StructProperty",
            "Struct in struct prop"
        );
        let field_size = r.read_u64()?;
        let field_name = r.read_uestring()?;
        let mut guid = [0u8; 16];
        r.read_exact(&mut guid)?;
        assert_eq!(guid, [0u8; 16], "Non-empty GUID");
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            return Err(
                Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into(),
            );
        }
        match field_name.as_str() {
            "Vector" => {
                assert_eq!(field_size % 12, 0, "Vector of the wrong size");
                assert_eq!(field_size, struct_size as u64 * 12, "Mismatched size");
                let mut data = Vec::with_capacity(field_size as usize / 12);
                for _ in 0..field_size / 12 {
                    data.push([r.read_f32()?, r.read_f32()?, r.read_f32()?]);
                }
                Ok(Self::VectorArray(data))
            }
            "Rotator" => {
                assert_eq!(field_size % 12, 0, "Rotator of the wrong size");
                assert_eq!(field_size, struct_size as u64 * 12, "Mismatched size");
                let mut data = Vec::with_capacity(field_size as usize / 12);
                for _ in 0..field_size / 12 {
                    data.push([r.read_f32()?, r.read_f32()?, r.read_f32()?]);
                }
                Ok(Self::RotatorArray(data))
            }
            _ => todo!("struct type {}", field_name),
        }
    }

    pub fn read_str_array(r: &mut impl Read, _plen: u64) -> Result<Self> {
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            return Err(
                Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into(),
            );
        }
        let ntext = r.read_u32()?;
        let mut data = Vec::with_capacity(ntext as usize);
        for _ in 0..ntext {
            data.push(r.read_uestring()?);
        }
        Ok(Self::StringArray(data))
    }

    pub fn read_text_array(r: &mut impl Read, _plen: u64) -> Result<Self> {
        let ch_bool = r.read_u8()? == 0;
        if !ch_bool {
            return Err(
                Error::new(ErrorKind::InvalidData, "Check bool != 0 is not implemented").into(),
            );
        }
        let ntext = r.read_u32()?;
        let mut data = Vec::with_capacity(ntext as usize);
        for _ in 0..ntext {
            data.push(TextProperty::read(r)?);
        }
        Ok(Self::TextArray(data))
    }
}

impl<'a> TryInto<&'a Vec<f32>> for &'a Value {
    type Error = GVASError;
    fn try_into(self) -> Result<&'a Vec<f32>> {
        match self {
            Value::FloatArray(f) => Ok(&f),
            _ => Err(GVASError::WrongType),
        }
    }
}

impl<'a> TryInto<&'a Vec<u32>> for &'a Value {
    type Error = GVASError;
    fn try_into(self) -> Result<&'a Vec<u32>> {
        match self {
            Value::Int32Array(f) => Ok(&f),
            _ => Err(GVASError::WrongType),
        }
    }
}

impl<'a> TryInto<&'a Vec<bool>> for &'a Value {
    type Error = GVASError;
    fn try_into(self) -> Result<&'a Vec<bool>> {
        match self {
            Value::BoolArray(f) => Ok(&f),
            _ => Err(GVASError::WrongType),
        }
    }
}

impl<'a> TryInto<&'a Vec<[f32; 3]>> for &'a Value {
    type Error = GVASError;
    fn try_into(self) -> Result<&'a Vec<[f32; 3]>> {
        match self {
            Value::RotatorArray(f) => Ok(&f),
            Value::VectorArray(f) => Ok(&f),
            _ => Err(GVASError::WrongType),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextProperty {
    Simple(String),
    FmtStr(String, String),
    None,
}

impl TextProperty {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        let before_sep = r.read_u32()?;
        if before_sep == 1 {
            assert_eq!(r.read_u8()?, 3, "Fmt Str Format");
            assert_eq!(r.read_u64()?, 8, "Fmt Str Format");
            assert_eq!(r.read_u8()?, 0, "Fmt Str Format");
            assert_eq!(
                r.read_uestring()?,
                "56F8D27149CC5E2D12103BBEBFCA9097",
                "Fmt Str Format"
            );
            let fmt_str = r.read_uestring()?;
            assert_eq!(fmt_str, "{0}<br>{1}", "Fmt Str Format");
            assert_eq!(r.read_u32()?, 2, "Fmt Str Format");
            assert_eq!(r.read_uestring()?, "0", "Fmt Str Format");
            assert_eq!(r.read_u8()?, 4, "Fmt Str Format");
            assert_eq!(r.read_u32()?, 2, "Fmt Str Format");
            assert_eq!(r.read_i8()?, -1, "Fmt Str Format");
            let opt = r.read_u32()?;
            let first_line = if opt == 1 {
                r.read_uestring()?
            } else {
                "".into()
            };
            assert_eq!(r.read_uestring()?, "1", "Fmt Str Format");
            assert_eq!(r.read_u8()?, 4, "Fmt Str Format");
            assert_eq!(r.read_u32()?, 2, "Fmt Str Format");
            assert_eq!(r.read_i8()?, -1, "Fmt Str Format");
            let opt = r.read_u32()?;
            let second_line = if opt == 1 {
                r.read_uestring()?
            } else {
                "".into()
            };
            Ok(Self::FmtStr(first_line, second_line))
        } else {
            assert_eq!(r.read_i8()?, -1, "");
            let opt = r.read_u32()?;
            if opt == 1 {
                Ok(Self::Simple(r.read_uestring()?))
            } else {
                Ok(Self::None)
            }
        }
    }

    pub fn write(&self, w: &mut impl Write) -> Result<u64> {
        Ok(match self {
            Self::None => {
                w.write_all(&0u32.to_le_bytes())?;
                w.write_all(&(-1i8).to_le_bytes())?;
                w.write_all(&0u32.to_le_bytes())?;
                9
            }
            Self::Simple(s) => {
                w.write_all(&2u32.to_le_bytes())?;
                w.write_all(&(-1i8).to_le_bytes())?;
                w.write_all(&1u32.to_le_bytes())?;
                w.write_string(s.as_str())?;
                9 + s.len() as u64 + 5
            }
            Self::FmtStr(first, second) => {
                w.write_all(&1u32.to_le_bytes())?;
                w.write_all(&3u8.to_le_bytes())?;
                w.write_all(&8u64.to_le_bytes())?;
                w.write_all(&0u8.to_le_bytes())?;
                let len = 14;
                w.write_string("56F8D27149CC5E2D12103BBEBFCA9097")?;
                let len = len + "56F8D27149CC5E2D12103BBEBFCA9097".len() as u64 + 5;
                w.write_string("{0}<br>{1}")?;
                let len = len + "{0}<br>{1}".len() as u64 + 5;
                w.write_all(&2u32.to_le_bytes())?;
                let len = len + 4;
                w.write_string("0")?;
                let len = len + "0".len() as u64 + 5;
                w.write_all(&4u8.to_le_bytes())?;
                let len = len + 1;
                w.write_all(&2u32.to_le_bytes())?;
                let len = len + 4;
                w.write_all(&(-1i8).to_le_bytes())?;
                let len = len + 1;
                let len = if first == "" {
                    w.write_all(&0u32.to_le_bytes())?;
                    len + 4
                } else {
                    w.write_all(&1u32.to_le_bytes())?;
                    w.write_string(first.as_str())?;
                    4 + first.len() as u64 + 5
                };
                w.write_string("1")?;
                let len = len + "1".len() as u64 + 5;
                w.write_all(&4u8.to_le_bytes())?;
                let len = len + 1;
                w.write_all(&2u32.to_le_bytes())?;
                let len = len + 4;
                w.write_all(&(-1i8).to_le_bytes())?;
                let len = len + 1;
                if second == "" {
                    w.write_all(&0u32.to_le_bytes())?;
                    len + 4
                } else {
                    w.write_all(&1u32.to_le_bytes())?;
                    w.write_string(second.as_str())?;
                    4 + second.len() as u64 + 5
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct RROSave {
    inner: GVASFile,
}

impl RROSave {
    pub fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self {
            inner: GVASFile::read(r)?,
        })
    }

    pub fn write(&self, r: &mut (impl Write + Seek)) -> Result<()> {
        self.inner.write(r)
    }

    pub fn curves<'a>(&'a self) -> Result<RROCurveIter<'a>> {
        Ok(RROCurveIter {
            i: 0,
            spline_location_array: self.inner.get_prop("SplineLocationArray")?.try_into()?,
            spline_type_array: self.inner.get_prop("SplineTypeArray")?.try_into()?,
            spline_control_points_array: self
                .inner
                .get_prop("SplineControlPointsArray")?
                .try_into()?,
            spline_control_points_index_start_array: self
                .inner
                .get_prop("SplineControlPointsIndexStartArray")?
                .try_into()?,
            spline_control_points_index_end_array: self
                .inner
                .get_prop("SplineControlPointsIndexEndArray")?
                .try_into()?,
            spline_segments_visibility_array: self
                .inner
                .get_prop("SplineSegmentsVisibilityArray")?
                .try_into()?,
            spline_visibility_start_array: self
                .inner
                .get_prop("SplineVisibilityStartArray")?
                .try_into()?,
            spline_visibility_end_array: self
                .inner
                .get_prop("SplineVisibilityEndArray")?
                .try_into()?,
        })
    }

    pub fn set_curves<'a>(&mut self, iter: impl Iterator<Item = CurveDataOwned>) -> Result<()> {
        let mut spline_location_array = vec![];
        let mut spline_type_array = vec![];
        let mut spline_control_points_array = vec![];
        let mut spline_control_points_index_start_array = vec![];
        let mut spline_control_points_index_end_array = vec![];
        let mut spline_segments_visibility_array = vec![];
        let mut spline_visibility_start_array = vec![];
        let mut spline_visibility_end_array = vec![];
        for curve in iter {
            spline_location_array.push(curve.location);
            spline_type_array.push(curve.ty as u32);
            spline_control_points_index_start_array.push(spline_control_points_array.len() as u32);
            for p in curve.control_points {
                spline_control_points_array.push(p);
            }
            spline_control_points_index_end_array
                .push(spline_control_points_array.len() as u32 - 1);
            spline_visibility_start_array.push(spline_segments_visibility_array.len() as u32);
            for p in curve.visibility {
                spline_segments_visibility_array.push(p);
            }
            spline_visibility_end_array.push(spline_segments_visibility_array.len() as u32 - 1);
        }
        *self.inner.get_prop_mut("SplineLocationArray")? =
            Value::VectorArray(spline_location_array);
        *self.inner.get_prop_mut("SplineTypeArray")? = Value::Int32Array(spline_type_array);
        *self.inner.get_prop_mut("SplineControlPointsArray")? =
            Value::VectorArray(spline_control_points_array);
        *self
            .inner
            .get_prop_mut("SplineControlPointsIndexStartArray")? =
            Value::Int32Array(spline_control_points_index_start_array);
        *self
            .inner
            .get_prop_mut("SplineControlPointsIndexEndArray")? =
            Value::Int32Array(spline_control_points_index_end_array);
        *self.inner.get_prop_mut("SplineSegmentsVisibilityArray")? =
            Value::BoolArray(spline_segments_visibility_array);
        *self.inner.get_prop_mut("SplineVisibilityStartArray")? =
            Value::Int32Array(spline_visibility_start_array);
        *self.inner.get_prop_mut("SplineVisibilityEndArray")? =
            Value::Int32Array(spline_visibility_end_array);
        Ok(())
    }
}

#[derive(Debug)]
pub struct CurveData<'a> {
    pub location: &'a [f32; 3],
    pub ty: SplineType,
    pub control_points: &'a [[f32; 3]],
    pub visibility: &'a [bool],
}

#[derive(Debug)]
pub struct CurveDataOwned {
    pub location: [f32; 3],
    pub ty: SplineType,
    pub control_points: Vec<[f32; 3]>,
    pub visibility: Vec<bool>,
}

pub struct RROCurveIter<'a> {
    i: usize,
    spline_location_array: &'a Vec<[f32; 3]>,
    spline_type_array: &'a Vec<u32>,
    spline_control_points_array: &'a Vec<[f32; 3]>,
    spline_control_points_index_start_array: &'a Vec<u32>,
    spline_control_points_index_end_array: &'a Vec<u32>,
    spline_segments_visibility_array: &'a Vec<bool>,
    spline_visibility_start_array: &'a Vec<u32>,
    spline_visibility_end_array: &'a Vec<u32>,
}

impl<'a> Iterator for RROCurveIter<'a> {
    type Item = CurveData<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.spline_location_array.len() {
            let ctrl_s = self.spline_control_points_index_start_array[self.i] as usize;
            let ctrl_e = self.spline_control_points_index_end_array[self.i] as usize;
            let vis_s = self.spline_visibility_start_array[self.i] as usize;
            let vis_e = self.spline_visibility_end_array[self.i] as usize;
            let curve = CurveData {
                location: &self.spline_location_array[self.i],
                ty: self.spline_type_array[self.i].try_into().expect("Invalid Spline Type"),
                control_points: &self.spline_control_points_array[ctrl_s..=ctrl_e],
                visibility: &self.spline_segments_visibility_array[vis_s..=vis_e],
            };
            self.i += 1;
            Some(curve)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.spline_location_array.len() - self.i,
            Some(self.spline_location_array.len() - self.i),
        )
    }
}

impl<'a> ExactSizeIterator for RROCurveIter<'a> {}

pub use scoped::*;

mod scoped {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, enum_utils::TryFromRepr)]
    #[repr(u32)]
    pub enum SplineType {
        Track = 0,
        TrackBed = 4,
        WoodBridge = 3,
        SteelBridge = 7,
        GroundWork = 1,
        ConstGroundWork = 2,
        StoneGroundWork = 5,
        ConstStoneGroundWork = 6,
    }
}
