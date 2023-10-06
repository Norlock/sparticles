use std::fmt::Formatter;
use std::ops::DerefMut;
use std::{num::NonZeroU64, ops::Deref};

use crate::traits::{CalculateBufferSize, HandleAngles};
use egui_wgpu::wgpu;
use glam::{Vec2, Vec3, Vec4};
use serde::de::{Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, SerializeSeq, Serializer};

impl HandleAngles for Vec3 {
    fn to_degrees(&self) -> Self {
        let x = self.x.to_degrees();
        let y = self.y.to_degrees();
        let z = self.z.to_degrees();

        Self { x, y, z }
    }

    fn to_radians(&self) -> Self {
        let x = self.x.to_radians();
        let y = self.y.to_radians();
        let z = self.z.to_radians();

        Self { x, y, z }
    }
}

impl HandleAngles for Vec2 {
    fn to_degrees(&self) -> Self {
        let x = self.x.to_degrees();
        let y = self.y.to_degrees();

        Self { x, y }
    }

    fn to_radians(&self) -> Self {
        let x = self.x.to_radians();
        let y = self.y.to_radians();

        Self { x, y }
    }
}

impl CalculateBufferSize for Vec<f32> {
    fn cal_buffer_size(&self) -> Option<NonZeroU64> {
        wgpu::BufferSize::new(self.len() as u64 * 4)
    }
}

impl CalculateBufferSize for [f32] {
    fn cal_buffer_size(&self) -> Option<NonZeroU64> {
        wgpu::BufferSize::new(self.len() as u64 * 4)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SparVec3(pub Vec3);
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SparVec4(pub Vec4);

impl Deref for SparVec3 {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SparVec4 {
    type Target = Vec4;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SparVec3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for SparVec4 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Serialize for SparVec3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        let _ = seq.serialize_element(&self.0.x);
        let _ = seq.serialize_element(&self.0.y);
        let _ = seq.serialize_element(&self.0.z);
        seq.end()
    }
}

impl From<Vec3> for SparVec3 {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

impl From<[f32; 3]> for SparVec3 {
    fn from(value: [f32; 3]) -> Self {
        Self(value.into())
    }
}

impl From<Vec4> for SparVec4 {
    fn from(value: Vec4) -> Self {
        Self(value)
    }
}

impl From<[f32; 4]> for SparVec4 {
    fn from(value: [f32; 4]) -> Self {
        Self(value.into())
    }
}

impl<'de> Deserialize<'de> for SparVec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Spar3Visitor)
    }
}

impl<'de> Deserialize<'de> for SparVec4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Spar4Visitor)
    }
}

impl Serialize for SparVec4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        let _ = seq.serialize_element(&self.0.x);
        let _ = seq.serialize_element(&self.0.y);
        let _ = seq.serialize_element(&self.0.z);
        let _ = seq.serialize_element(&self.0.w);
        seq.end()
    }
}

// Visitors
pub struct Spar3Visitor;
impl<'de> Visitor<'de> for Spar3Visitor {
    type Value = SparVec3;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Not a vec3")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let arr: Result<[f32; 3], _> = serde_json::from_str(v);

        if let Ok(arr) = arr {
            Ok(arr.into())
        } else {
            Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Seq,
                &self,
            ))
        }
    }
}

pub struct Spar4Visitor;
impl<'de> Visitor<'de> for Spar4Visitor {
    type Value = SparVec4;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Not a vec3")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let arr: Result<[f32; 4], _> = serde_json::from_str(v);

        if let Ok(arr) = arr {
            Ok(arr.into())
        } else {
            Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Seq,
                &self,
            ))
        }
    }
}
