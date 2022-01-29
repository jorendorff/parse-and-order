use std::str::FromStr;

use serde::{
    de::{EnumAccess, MapAccess, SeqAccess, VariantAccess},
    Deserializer,
};

use crate::error::MyError;
use crate::interact;

#[derive(Clone)]
pub struct InteractiveDe {
    indent_level: usize,
    prompt: String,
    identifiers: Vec<String>,
}

impl InteractiveDe {
    pub fn new(result_name: &str) -> Self {
        InteractiveDe {
            indent_level: 0,
            prompt: result_name.to_string(),
            identifiers: vec![],
        }
    }

    fn child(&self, suffix: &str) -> Self {
        InteractiveDe {
            indent_level: self.indent_level + 4,
            prompt: self.prompt.clone() + suffix,
            identifiers: vec![],
        }
    }

    fn child_with_identifier(&self, suffix: &str, ident: &str) -> Self {
        let mut child = self.child(suffix);
        child.identifiers.push(ident.to_string());
        child
    }

    fn read_line(&self, prompt: &str) -> Result<String, MyError> {
        let full_prompt = format!(
            "{:width$}{} {}> ",
            "",
            self.prompt,
            prompt,
            width = self.indent_level
        );
        interact::read_line(&full_prompt)
    }

    fn read<T>(&self) -> Result<T, MyError>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Display,
    {
        let full_prompt = format!(
            "{:width$}{} - Enter a {}> ",
            "",
            self.prompt,
            std::any::type_name::<T>(),
            width = self.indent_level
        );
        interact::read(&full_prompt)
    }

    fn confirm(&self, end: &str) -> Result<bool, MyError> {
        let full_prompt = format!(
            "{:width$}{} - {}? ",
            "",
            self.prompt,
            end,
            width = self.indent_level
        );
        interact::confirm(&full_prompt)
    }

    fn choose_one(&self, name: &str, variants: &[&str]) -> Result<usize, MyError> {
        println!("{:width$}Choose one of:", "", width = self.indent_level);
        for (i, v) in variants.iter().enumerate() {
            println!("{:width$}  {}. {}", "", i, *v, width = self.indent_level);
        }
        let full_prompt = format!(
            "{:width$}{} {}> ",
            "",
            self.prompt,
            name,
            width = self.indent_level
        );
        loop {
            let i = interact::read::<usize>(&full_prompt)?;
            if i < variants.len() {
                break Ok(i);
            }
            println!("  please choose a value in 0..{}", variants.len());
        }
    }
}

struct InteractiveTupleDe<'a> {
    parent: &'a InteractiveDe,
    index: usize,
    length: usize,
}

impl<'a, 'de> SeqAccess<'de> for InteractiveTupleDe<'a> {
    type Error = MyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.index < self.length {
            let prompt = format!(".{}", self.index);
            let de = self.parent.child(&prompt);
            let value = seed.deserialize(de)?;
            self.index += 1;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

struct InteractiveSeqDe<'a> {
    parent: &'a InteractiveDe,
    index: usize,
}

impl<'a, 'de> SeqAccess<'de> for InteractiveSeqDe<'a> {
    type Error = MyError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let prompt = if self.index == 0 {
            "Any elements to add"
        } else {
            "Add another element"
        };
        if self.parent.confirm(prompt)? {
            let prompt = format!("[{}]", self.index);
            let de = self.parent.child(&prompt);
            let value = seed.deserialize(de)?;
            self.index += 1;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

struct InteractiveStructDe<'a> {
    parent: &'a InteractiveDe,
    index: usize,
    fields: &'a [&'a str],
}

impl<'a, 'de> MapAccess<'de> for InteractiveStructDe<'a> {
    type Error = MyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if let Some(field) = self.fields.get(self.index) {
            let de = self
                .parent
                .child_with_identifier(&format!(".{}", *field), *field);
            let value = seed.deserialize(de)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let de = self.parent.child(&format!(".{}", self.fields[self.index]));
        self.index += 1;
        seed.deserialize(de)
    }
}

struct InteractiveMapDe<'a> {
    parent: &'a InteractiveDe,
    index: usize,
}

impl<'a, 'de> MapAccess<'de> for InteractiveMapDe<'a> {
    type Error = MyError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let prompt = if self.index == 0 {
            "Any map entries to add"
        } else {
            "Add another map entry"
        };
        if self.parent.confirm(prompt)? {
            let de = self.parent.child(&format!(".entries[{}].key", self.index));
            let value = seed.deserialize(de)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let prompt = format!(".entries[{}].value", self.index);
        self.index += 1;
        let de = self.parent.child(&prompt);
        seed.deserialize(de)
    }
}

struct InteractiveEnumDe<'a> {
    parent: &'a InteractiveDe,
    name: &'a str,
    variants: &'a [&'a str],
}

impl<'a, 'de> EnumAccess<'de> for InteractiveEnumDe<'a> {
    type Error = MyError;

    type Variant = InteractiveVariantDe<'a>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let i = self.parent.choose_one(self.name, self.variants)?;

        let value = seed.deserialize(self.parent.child_with_identifier("", self.variants[i]))?;
        Ok((
            value,
            InteractiveVariantDe {
                parent: self.parent,
                enum_name: self.name,
                variant_name: self.variants[i],
            },
        ))
    }
}

struct InteractiveVariantDe<'a> {
    parent: &'a InteractiveDe,
    enum_name: &'a str,
    variant_name: &'a str,
}

impl<'a, 'de> VariantAccess<'de> for InteractiveVariantDe<'a> {
    type Error = MyError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(
            self.parent
                .child(&format!("(as {}::{})", self.enum_name, self.variant_name)),
        )
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(InteractiveTupleDe {
            parent: self.parent,
            index: 0,
            length: len,
        })
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(InteractiveStructDe {
            parent: self.parent,
            index: 0,
            fields,
        })
    }
}

impl<'de> Deserializer<'de> for InteractiveDe {
    type Error = MyError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(self.read()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(self.read()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i16(self.read()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(self.read()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i64(self.read()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.read()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(self.read()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(self.read()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u64(self.read()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f32(self.read()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f64(self.read()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.read_line("String")?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.confirm("option? ")? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        println!(
            "{:width$}{} - Enter a ()> (no input required)",
            "",
            self.prompt,
            width = self.indent_level
        );
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        println!(
            "{:width$}{} - Enter a {}> (no input required)",
            "",
            self.prompt,
            name,
            width = self.indent_level
        );
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self.child(".0"))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(InteractiveSeqDe {
            parent: &self,
            index: 0,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(InteractiveTupleDe {
            parent: &self,
            index: 0,
            length: len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(InteractiveTupleDe {
            parent: &self,
            index: 0,
            length: len,
        })
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(InteractiveMapDe {
            parent: &self,
            index: 0,
        })
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        println!("{:width$}struct {} {{", "", name, width = self.indent_level);
        for field in fields {
            println!("{:width$}    {},", "", *field, width = self.indent_level);
        }
        println!("{:width$}}}", "", width = self.indent_level);

        visitor.visit_map(InteractiveStructDe {
            parent: &self,
            index: 0,
            fields,
        })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(InteractiveEnumDe {
            parent: &self,
            name,
            variants,
        })
    }

    fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let ident = if let Some(ident) = self.identifiers.pop() {
            ident
        } else {
            self.read_line("identifier")?
        };
        visitor.visit_string(ident)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}
