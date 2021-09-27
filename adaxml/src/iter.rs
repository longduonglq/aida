use crate::tag::XmlTag;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::rc::{Rc, Weak};
use std::array::IntoIter;

pub struct BfsXmlTagIter {
    queue: VecDeque<Rc<XmlTag>>
}

impl<T> From<T> for BfsXmlTagIter
where
    T: Into<Rc<XmlTag>>
{
    fn from(tag: T) -> Self {
        Self {
            queue: VecDeque::from_iter([tag.into()]),
        }
    }
}

impl Iterator for BfsXmlTagIter {
    type Item = Rc<XmlTag>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.queue.is_empty() {
            let v = self.queue.pop_front().unwrap();
            for child in &v.children {
                self.queue.push_back(child.clone());
            }
            return Some(v);
        }
        None
    }
}


pub struct DfsXmlTagIter {
    stack: Vec<Rc<XmlTag>>
}

impl<T> From<T> for DfsXmlTagIter
    where
    T: Into<Rc<XmlTag>>
{
    fn from(tag: T) -> Self {
        Self {
            stack: Vec::from_iter([tag.into()]),
        }
    }
}

impl Iterator for DfsXmlTagIter {
    type Item = Rc<XmlTag>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.stack.is_empty() {
            let v = self.stack.pop().unwrap();
            for child in v.children.iter().rev() {
                self.stack.push(child.clone());
            }
            return Some(v);
        }
        None
    }
}

#[cfg(test)]
mod tests{
    use crate::tag::XmlTag;
    use crate::io;
    use crate::iter::{BfsXmlTagIter, DfsXmlTagIter};

    #[test]
    fn test_bfs_iter() {
        let tree = XmlTag::from_path("test/template.musicxml").unwrap();
        for leaf in BfsXmlTagIter::from(tree) {
            leaf.show_local_tag();
        }
    }

    #[test]
    fn test_dfs_iter() {
        let tree = XmlTag::from_path("test/template.musicxml").unwrap();
        for leaf in DfsXmlTagIter::from(tree) {
            leaf.show_local_tag();
        }
    }
}