use crate::tag::XmlTag;
use std::collections::VecDeque;
use std::iter::FromIterator;

struct BfsXmlTagIter<'a> {
    queue: VecDeque<&'a XmlTag>
}

impl<'a> From<&'a XmlTag> for BfsXmlTagIter<'a> {
    fn from(tag: &'a XmlTag) -> Self {
        Self {
            queue: VecDeque::from_iter([tag]),
        }
    }
}

impl<'a> Iterator for BfsXmlTagIter<'a> {
    type Item = &'a XmlTag;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.queue.is_empty() {
            let v = self.queue.pop_front().unwrap();
            for child in &v.children {
                self.queue.push_back(child);
            }
            return Some(&v);
        }
        None
    }
}

struct DfsXmlTagIter<'a> {
    stack: Vec<&'a XmlTag>
}

impl<'a> Iterator for DfsXmlTagIter<'a> {
    type Item = &'a XmlTag;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
