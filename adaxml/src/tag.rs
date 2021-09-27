use std::collections::VecDeque;
use std::iter::Iterator;
use std::path::Iter;
use xml::common::XmlVersion;
use chrono::{Date, Utc, DateTime};
use std::borrow::Cow;
use std::rc::Rc;
use super::iter::*;

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
pub struct XmlTag {
    pub name: XmlString,
    pub value: Option<XmlString>,
    pub attribs: Vec<XmlAttrib>,
    pub children: Vec< Rc<XmlTag>>
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
    fn all_child_with_name(self: Rc<XmlTag>, name: &'static str)
        -> impl Iterator<Item = Rc<XmlTag>>
    {
        let iter = BfsXmlTagIter::from(self);
        iter.filter(move |tag| {tag.name.as_str() == name})
    }

    fn first_child_with_name(self: Rc<XmlTag>, name: &'static str)
                             -> impl Iterator<Item = Rc<XmlTag>>
    {
        self.all_child_with_name(name).take(1)
    }


    // Tag info

    // Private

}


#[cfg(test)]
mod tests {
    use crate::tag::XmlTag;

    #[test]
    fn load_tree() {
        let tree = XmlTag::from_path("test/template.musicxml").unwrap();
        for child in tree.first_child_with_name("part") {
            println!("{:?}", child);
        }
    }
}