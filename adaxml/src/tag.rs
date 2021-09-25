use std::collections::VecDeque;
use std::iter::Iterator;
use std::path::Iter;

type XmlString = String;

#[derive(Clone, Debug)]
pub struct XmlMetaData {
}

#[derive(Clone, Debug)]
pub struct XmlAttrib {
    name: XmlString,
    value: Option<XmlString>
}

#[derive(Clone, Debug)]
pub struct XmlTag {
    name: XmlString,
    value: Option<XmlString>,
    attribs: Vec<XmlAttrib>,
    children: Vec<Box<XmlTag>>
}

impl<'a> Iterator for XmlTag {
    type Item = &'a XmlTag;

    fn next(&'a mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl XmlTag
{
    // Public functions
    fn children_with_name(name: &str) {

    }
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
        self.get_tag_st(
            |tag: &XmlTag| {
                selector(
                    &tag.name,
                    &tag.attribs,
                    &tag.children
                )
            }
        )
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn load_tree() {

    }
}