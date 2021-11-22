use std::collections::VecDeque;
use std::iter::Iterator;
use std::path::Iter;
use xml::common::XmlVersion;
use chrono::{Date, Utc, DateTime};
use super::iter::*;
use std::str::FromStr;
use xml::attribute::Attribute;

pub type XmlString = String;

#[derive(Clone, Debug)]
pub struct XmlMetaData {
    pub version: XmlVersion,
    pub encoding: String,
    pub date: Date<Utc>
}

#[derive(Clone, Debug)]
pub struct XmlAttrib {
    pub name: XmlString,
    pub value: XmlString
}

#[derive(Clone)]
pub struct XmlTag {
    pub name: XmlString,
    pub value: Option<XmlString>,
    pub attribs: Vec<XmlAttrib>,
    pub children: Vec<XmlTag>
}

impl Default for XmlTag {
    fn default() -> Self {
        XmlTag {
            name: XmlString::new(),
            value: None,
            attribs: Vec::with_capacity(5),
            children: Vec::with_capacity(50)
        }
    }
}

impl Default for XmlMetaData {
    fn default() -> Self {
        XmlMetaData {
            version: XmlVersion::Version10,
            encoding: "".to_string(),
            date: Utc::now().date()
        }
    }
}

impl XmlTag
{
    // Public functions

    // Search
    pub fn all_desc_with_name(&self, name: &'static str)
        -> impl Iterator<Item=&XmlTag>
    {
        BfsXmlTagIter::from(self)
        .filter(move |tag| {tag.name == name})

    }

    pub fn first_desc_with_name(&self, name: &'static str)
        -> impl Iterator<Item=&XmlTag>
    {
        self.all_desc_with_name(name).take(1)
    }

    pub fn all_child_with_name(&self, name: &'static str)
        -> impl Iterator<Item=&XmlTag>
    {
        self.children.iter()
        .filter(move |child| {child.name == name})
    }

    pub fn first_child_with_name(&self, name: &'static str)
        -> Option<&XmlTag>
    {
        self.all_child_with_name(name)
        .next()
    }

    pub fn all_attribs_with_name(&self, name: &'static str)
        -> impl Iterator<Item=&XmlAttrib>
    {
        self.attribs.iter()
        .filter(move |attr| {attr.name == name})
    }

    pub fn all_child_with_attrib(&self, name: &'static str, value: &'static str)
        -> impl Iterator<Item=&XmlTag>
    {
        self.children.iter()
        .filter(
            move |ch| {
                ch.attribs.iter()
                .any(|attr| {attr.name == name && attr.value == value})
            }
        )
    }

    pub fn get_attrib_value(&self, name: &'static str)
        -> Option<&str>
    {
        self.all_attribs_with_name(name).next()
        .map(|attr| {attr.value.as_str()})
    }

    pub fn get_attrib_value_as<T: FromStr>(&self, name: &'static str)
        -> Option<T>
    {
        self.get_attrib_value(name)
        .map(|val| {val.parse().ok()})
        .flatten()
    }

    pub fn get_child_with_attrib(&self, name: &'static str, value: &'static str)
        -> Option<&XmlTag>
    {
        self.all_child_with_attrib(name, value).next()
    }


    /// Builder methods

    pub fn add_attribute(&mut self, name: XmlString, value: XmlString) -> &mut Self {
        self.attribs.push(
            XmlAttrib {
                name,
                value
            }
        );
        self
    }

    pub fn add_attribute_with_type<T: ToString>(&mut self, name: XmlString, value: T)
        -> &mut Self
    {
        self.add_attribute(name, value.to_string());
        self
    }

    pub fn add_child(&mut self, name:  XmlString) -> &mut XmlTag {
        self.children.push(
            XmlTag {
                name,
                value: None,
                attribs: vec![],
                children: vec![]
            }
        );
        self.children.last_mut().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use crate::tag::XmlTag;
    use std::rc::Rc;

    #[test]
    fn load_tree() {
        let mut tree = XmlTag::from_path("test/template.musicxml").unwrap();
        for child in tree.all_child_with_name("identification") {
            println!("{:?} -- {:?}", child, child.get_attrib_value("id").unwrap_or("NOTHING"));
        }
        let mut subtree = tree.first_desc_with_name("identification").last().unwrap();
        let mut mutsubtree = subtree.to_owned();
        mutsubtree.name = "studp".to_string();
        mutsubtree.value = Some("Long".to_string());
        mutsubtree.children[0].name = "fs".to_string();
        println!("{:?}", subtree);
        println!("{:?}", mutsubtree);
    }
}