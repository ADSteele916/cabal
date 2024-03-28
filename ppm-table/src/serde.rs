use std::fmt;
use std::hash::BuildHasher;
use std::marker::PhantomData;

use serde::de::{Deserializer, Error, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use crate::PpmTable;

#[cfg(feature = "serde")]
impl<R: BuildHasher + Default> Serialize for PpmTable<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PpmTable", 2)?;
        state.serialize_field("ppm_table", &self.ppm_table)?;
        state.serialize_field("indices", &self.indices)?;
        state.end()
    }
}

impl<'de, R: BuildHasher + Default> Deserialize<'de> for PpmTable<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            PpmTable,
            Indices,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`ppm_table` or `indices`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: Error,
                    {
                        match value {
                            "ppm_table" => Ok(Field::PpmTable),
                            "indices" => Ok(Field::Indices),
                            _ => Err(Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PpmTableVisitor<R: BuildHasher + Default> {
            phantom: PhantomData<R>,
        }

        impl<'de, R: BuildHasher + Default> Visitor<'de> for PpmTableVisitor<R> {
            type Value = PpmTable<R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PpmTable")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let ppm_table = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let indices = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(1, &self))?;
                Ok(PpmTable { ppm_table, indices })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut ppm_table = None;
                let mut indices = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::PpmTable => {
                            if ppm_table.is_some() {
                                return Err(Error::duplicate_field("ppm_table"));
                            }
                            ppm_table = Some(map.next_value()?);
                        }
                        Field::Indices => {
                            if indices.is_some() {
                                return Err(Error::duplicate_field("indices"));
                            }
                            indices = Some(map.next_value()?);
                        }
                    }
                }
                let ppm_table = ppm_table.ok_or_else(|| Error::missing_field("ppm_table"))?;
                let indices = indices.ok_or_else(|| Error::missing_field("indices"))?;
                Ok(PpmTable { ppm_table, indices })
            }
        }

        const FIELDS: &[&str] = &["ppm_table", "indices"];
        deserializer.deserialize_struct(
            "PpmTable",
            FIELDS,
            PpmTableVisitor::<R> {
                phantom: Default::default(),
            },
        )
    }
}
