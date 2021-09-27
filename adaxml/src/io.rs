use super::tag::*;
use xml::reader::{self, XmlEvent::*, Events, XmlEvent};
use core::iter::Peekable;
use std::io::Read;
use xml::attribute::Attribute;
use std::fmt::{Debug, Formatter, Write};
use std::rc::Rc;
use std::borrow::{BorrowMut, Borrow};

impl XmlTag
{
    fn from_reader<T: Read>(mut events: Peekable<Events<T>>)
        -> (Option<XmlMetaData>, Option<Rc<XmlTag>>)
    {
        // Take care of StartDocument here
        fn _recursive_build<T: Read>(events: &mut Peekable<Events<T>>)
            -> Option<Rc<XmlTag>>
        {
            /**
            Working mechanism:
            1. pull tag, prefill info to XmlTag.
            2. peek next tag.
                * if next tag closes the current one then return vec
                * if next tag is something else, DON'T pull, push to vec the result of _recursive_build
                on the next tag.
                after doing this, peek again to see if we can close ourselves, if not then continue until
                we can close ourself and return

            Note: if _recursive_build returns Vec of length 1 containing Characters then incorporate that
            into the value field of the parent's Tag for convenience
             */
            let mut current_tag: XmlTag = Default::default();
            let mut children: Vec<_> = Vec::with_capacity(10);
            let pull_result = events.next();
            if pull_result.is_none() {
                return None;
            }

            let current_event = pull_result.unwrap();
            match current_event {
                Ok(StartElement {name, attributes, ..}) => {
                    current_tag = XmlTag {
                        name: name.local_name.to_string(),
                        value: None,
                        attribs: {
                            let mut xml_attribs = Vec::new();
                            for xml_attrib in attributes {
                                xml_attribs.push (
                                    XmlAttrib {
                                        name: xml_attrib.name.local_name.to_string(),
                                        value: xml_attrib.value.to_string()
                                    }
                                )
                            }
                            xml_attribs
                        },
                        children: Vec::with_capacity(10)
                    };
                }
                Ok(Characters(value)) | Ok(CData(value)) => {
                    current_tag = XmlTag {
                        value: {if value.is_empty() {None} else {Some(value)}},
                        ..current_tag
                    };
                    return Some(Rc::new(current_tag));
                }
                Ok(EndElement {name}) => {
                    // Logical error because EndElement should be taken care of already when processing
                    // the StartElement. In short, _recursive_build process EndElement instead of
                    // calling itself to process this.
                    unreachable!()
                }
                Ok(StartDocument {..}) => {
                    return _recursive_build(events); // passthrough
                }
                Ok(EndDocument) => {
                    return None;
                }
                Ok(ProcessingInstruction {..}) | Ok(Comment(_)) | Ok(Whitespace(_)) => {
                    return None;
                }
                _ => {unreachable!()}
            }
            // Now we focus on getting all children tag
            // Loop thru all child until the EndElement tag with our name is found
            while
                events.peek().is_some() &&
                {
                    let xml_event = events.peek().unwrap().as_ref().unwrap();
                    match xml_event {
                        EndElement{name} => {
                            if current_tag.name == name.local_name.as_str() { false }
                            else { false } // happens when tag closes immediately <tag></tag>
                        }
                        _ => true
                    }
                }
            {
                let mut build_res = _recursive_build(events);
                if build_res.is_some()
                {
                    let built_child = build_res.as_ref().unwrap();
                    if built_child.name.is_empty() && built_child.value.is_some() {
                        current_tag.value = build_res.as_ref().unwrap().value.clone();
                    }
                    else {
                        children.push(build_res.unwrap());
                    }
                }
            }
            if events.peek().is_some() {
                let next_event = events.peek();
                assert_eq!(
                    current_tag.name,
                    {
                        match next_event.unwrap().as_ref().unwrap() {
                            EndElement {name} => {name.local_name.as_str()},
                            _ => unreachable!()
                        }
                    }
                );
                events.next();
            } // removing EndElement(name)
            current_tag.children = children;
            return Some(Rc::new(current_tag));
        }
        (Some(XmlMetaData::default()), _recursive_build(&mut events))
    }
}

impl Debug for XmlTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        static ATOMIC_INDENT: &'static str = "    ";
        fn _recurse_write(me: &XmlTag, depth: i32, sink: &mut Formatter<'_>) {
            let indent =
            (0..depth).map(|_| {ATOMIC_INDENT})
            .fold(String::with_capacity(5), |r, s| r + s);

            sink.write_fmt(format_args!(
                "{0}+<{1} {2}> {3}\n",
                indent, me.name,
                me.attribs.iter()
                    .map(|attr: &XmlAttrib| {format!("{}={}", attr.name, attr.value)})
                    .fold("".to_string(), |r, s| {r + s.as_str() + " "}),
                {
                    if me.value.is_some() {me.value.as_ref().unwrap()}
                    else {""}
                }
            ));
            for tag in me.children.iter() {
                _recurse_write(tag, depth + 1, sink);
            }
            sink.write_fmt(format_args!(
                "{0}-</{1}> \n",
                indent,
                me.name
            ));
        }
        _recurse_write(self, 0, f);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use xml::{EventReader, ParserConfig};
    use crate::tag::XmlTag;
    use std::fs::File;

    #[test]
    fn exp(){
        let mut f = File::open("test/template.musicxml").unwrap();
        let mut reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true)
        .create_reader(f);
        // for event in reader.into_iter().peekable() {
        //     println!("{:?}\n", event);
        // }
        let tree =
        XmlTag::from_reader(reader.into_iter().peekable());
        println!("{:?}", tree.1.unwrap());
    }
}