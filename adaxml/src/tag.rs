use std::collections::VecDeque;
use std::iter::Iterator;
use std::path::Iter;
use xml::common::XmlVersion;
use chrono::{Date, Utc, DateTime};
use std::borrow::{Cow, Borrow};
use super::iter::*;
use std::str::FromStr;

type XmlString = String;

#[derive(Clone, Debug)]
pub struct XmlMetaData {
    version: XmlVersion,
    encoding: String,
    date: Date<Utc>
}

#[derive(Clone, Debug)]
pub struct XmlAttrib {
    pub name: XmlString,
    pub value: XmlString
}

#[derive(Clone)]
pub struct XmlTag<'a> {
    pub name: XmlString,
    pub value: Option<XmlString>,
    pub attribs: Vec<XmlAttrib>,
    pub children: Vec< Cow<'a, XmlTag<'a>>>
}

impl<'a> Default for XmlTag<'a> {
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

impl<'a> XmlTag<'a>
{
    // Public functions

    // Search
    fn all_child_with_name(&'a self, name: &'static str)
        -> impl Iterator<Item = Cow<'a, XmlTag<'a>>>
    {
        let iter = BfsXmlTagIter::from(Cow::Borrowed(self));
        iter.filter(move |tag| {tag.name.as_str() == name})
    }

    fn first_child_with_name(&'a self, name: &'static str)
        -> impl Iterator<Item = Cow<'a, XmlTag<'a>>>
    {
        self.all_child_with_name(name).take(1)
    }

    // Tag info
    fn get_attrib_value<T>(&self, key: &'static str)
        -> Option<T>
    where
        T: std::str::FromStr
    {
        self.attribs.iter()
        .filter(|attr| {attr.name.as_str() == key})
        .next()
        .map(|tag| {tag.value.clone()})
        .map(|val| { val.parse().ok() })
        .flatten()
    }
    // Private

}


#[cfg(test)]
mod tests {
    use crate::tag::XmlTag;
    use std::rc::Rc;

    #[test]
    fn load_tree() {
        let mut tree = XmlTag::from_path("test/template.musicxml").unwrap();
        for child in tree.first_child_with_name("identification") {
            println!("{:?} -- {:?}", child, child.get_attrib_value("id").unwrap_or("NOTHING".to_string()));
        }
        let mut subtree = tree.first_child_with_name("identification").last().unwrap();
        let mut mutsubtree = subtree.to_owned();
        mutsubtree.to_mut().name = "studp".to_string();
        mutsubtree.to_mut().value = Some("Long".to_string());
        mutsubtree.to_mut().children[0].to_mut().name = "fs".to_string();
        println!("{:?}", subtree);
        println!("{:?}", mutsubtree);
    }
}