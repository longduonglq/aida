use std::collections::VecDeque;
use std::iter::Iterator;
use std::path::Iter;
use xml::common::XmlVersion;
use chrono::{Date, Utc, DateTime};
use std::borrow::Cow;
use std::rc::Rc;

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
    // fn subtags_with_name(name: &str) -> impl Iterator<Item = &XmlTag> {
    //     todo!();
    // }

    // Private
    fn get_tag_st<F: Fn(&XmlTag) -> bool>(
        &self,
        cond: F
    ) -> Option<&XmlTag>
    {
        let mut queue = VecDeque::new();
        queue.push_back(self);
        while !queue.is_empty() {
            let v = queue.pop_front().unwrap();
            if !cond(&v) { return Some(&v); }
            for child in &v.children {
                queue.push_back(child);
            }
        }
        None
    }

    fn find_first_tag_st(
        &self,
        selector: fn(
            name: &str,
            attribs: &Vec<XmlAttrib>,
            children: &Vec<Box<XmlTag>>
        ) -> bool
    ) -> Option<&XmlTag>
    {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn load_tree() {

    }
}