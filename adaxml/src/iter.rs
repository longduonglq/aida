use crate::tag::{XmlTag};
use std::collections::VecDeque;

macro_rules! __cast_const_to_mut_ref {
    ($ref:expr, $type:ty) => {
        (&mut *(($ref as *const $type) as *mut $type))
    };
}

macro_rules! __cast_mut_to_const_ref {
    ($ref:expr, $type:ty) => {
        (& *(($ref as *mut $type) as *const $type))
    };
}

// BFS
pub struct BfsXmlTagIter<'a> {
    queue: VecDeque<&'a XmlTag>
}

impl<'a> From<&'a XmlTag> for BfsXmlTagIter<'a>
{
    // Require mut ref to make sure the user has mutable access
    fn from(tag: &'a XmlTag) -> Self {
        Self { queue : VecDeque::from([tag])}
    }
}

impl<'a> Iterator for BfsXmlTagIter<'a> {
    type Item = &'a XmlTag;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.queue.pop_front() {
            self
            .queue
            .extend(v.children.iter());

            Some(v)
        } else { None }
    }
}

// DFS
pub struct DfsXmlTagIter<'a> {
    stack: Vec<&'a XmlTag>
}

impl<'a> From<&'a XmlTag> for DfsXmlTagIter<'a>
{
    fn from(tag: &'a XmlTag) -> Self {
        Self {
            stack: vec![tag],
        }
    }
}

impl<'a> Iterator for DfsXmlTagIter<'a> {
    type Item = &'a XmlTag;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.stack.pop() {
            self
            .stack
            .extend(
                v
                .children
                .iter()
                .rev()
            );

            Some(v)
        } else { None }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use crate::tag::XmlTag;
    use crate::io;
    use crate::iter::{*};

    #[test]
    fn test_bfs_iter() {
        let tree = XmlTag::from_path("test/template.musicxml").unwrap();
        for leaf in BfsXmlTagIter::from(tree.borrow()) {
            leaf.show_local_tag();
        }
    }

    #[test]
    fn test_dfs_iter() {
        let tree = XmlTag::from_path("test/template.musicxml").unwrap();
        for leaf in DfsXmlTagIter::from(tree.borrow()) {
            leaf.show_local_tag();
        }
    }
}