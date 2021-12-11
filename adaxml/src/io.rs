pub use super::tag::*;
use xml::reader::{XmlEvent::*, Events};
use core::iter::Peekable;
use std::borrow::Cow;
use std::io::{Read, Write};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use anyhow::Error;
use xml::writer::{EventWriter, EmitterConfig, XmlEvent as WriterXmlEvent};
use xml::{ParserConfig};
use xml::common::XmlVersion;
use xml::namespace::Namespace;

#[derive(Debug)]
enum AdaIoErr {
    OpenFileErr,
    ParseFileErr
}

impl Display for AdaIoErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(
            format_args!("{}",
            match self {
                Self::OpenFileErr => "OpenFileErr",
                Self::ParseFileErr => "ParseFileErr"
            })
        )
    }
}

impl std::error::Error for AdaIoErr {}

impl XmlTag
{
    pub fn from_reader<T: Read>(mut events: Peekable<Events<T>>)
        -> anyhow::Result<XmlTag>
    {
        // TODO: Take care of StartDocument here
        fn _recursive_build<T: Read>(events: &mut Peekable<Events<T>>)
            -> Option<XmlTag>
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
            if pull_result.is_none() { return None; }

            let current_event = pull_result.unwrap().unwrap();
            match current_event {
                StartElement {name, attributes, ..} => {
                    current_tag = XmlTag {
                        name: name.local_name.to_string(),
                        value: None,
                        attribs: {
                            let mut xml_attribs = Vec::new();
                            xml_attribs
                            .extend(
                                attributes
                                .iter()
                                .map(
                                    |xml_attr| {
                                        XmlAttrib {
                                            name: xml_attr.name.local_name.to_string(),
                                            value: xml_attr.value.to_string()
                                        }
                                    }
                                )
                            );
                            xml_attribs
                        },
                        children: Vec::with_capacity(10)
                    };
                }
                Characters(value) | CData(value) => {
                    current_tag = XmlTag {
                        value: {if value.is_empty() {None} else {Some(value)}},
                        ..current_tag
                    };
                    return Some(current_tag);
                }
                EndElement {..} => {
                    // Logical error because EndElement should be taken care of already when processing
                    // the StartElement. In short, _recursive_build process EndElement instead of
                    // calling itself to process this.
                    unreachable!()
                }
                StartDocument {..} => {
                    return _recursive_build(events); // passthrough
                }
                // These terminal tag can be ignored by returning None;
                // However, the top level StartDocument must be passed through by returning recursive call.
                EndDocument => { return None; }
                ProcessingInstruction {..} | Comment(_) | Whitespace(_) => {
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
                let build_res = _recursive_build(events);
                // Ignore terminal ignorable tags (ie CData, Whitespace,... )
                if let Some(built_child) = build_res
                {
                    if built_child.name.is_empty() && built_child.value.is_some() {
                        current_tag.value = built_child.value;
                    }
                    else {
                        children.push(built_child);
                    }
                }
            }
            if let Some(next_event) = events.peek() {
                assert_eq!(
                    current_tag.name,
                    {
                        match &next_event {
                            Ok(EndElement {name}) => {name.local_name.as_str()},
                            _ => unreachable!()
                        }
                    }
                );
                events.next();
            } // removing EndElement(name)
            current_tag.children = children;
            return Some(current_tag);
        }
        _recursive_build(&mut events).ok_or(Error::from(AdaIoErr::ParseFileErr))
    }

    pub fn from_path(path: &str) -> anyhow::Result<XmlTag>
    {
        let f = File::open(path)?;
        let reader = ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true)
            .create_reader(f);
        let tree = XmlTag::from_reader(reader.into_iter().peekable());
        tree
    }

    pub fn to_writer<W: Write>(&self, writer: &mut EventWriter<W>) {
        fn _recursive_write<W: Write>(me: &XmlTag, w: &mut EventWriter<W>) {
            // TODO: quite stupid, please remove
            let DUMMY_NAMESPACE: Namespace = xml::namespace::Namespace::empty();
            let tag_begin = WriterXmlEvent::StartElement {
                name: xml::name::Name {
                    local_name: me.name.as_str(),
                    namespace: None,
                    prefix: None
                },
                attributes: {
                    me
                    .attribs
                    .iter()
                    .map(|attrib|{
                        xml::attribute::Attribute {
                            name: xml::name::Name::from(attrib.name.as_str()),
                            value: attrib.value.as_ref()
                        }
                    })
                    .collect()
                },
                namespace: Cow::Borrowed(&DUMMY_NAMESPACE)
            };
            w.write(tag_begin);
            if me.value.is_some() {
                let value_event = WriterXmlEvent::Characters(me.value.as_ref().unwrap());
                w.write(value_event).unwrap();
            }

            for child in me.children.iter() {
                _recursive_write(child, w)
            }

            let tag_end = WriterXmlEvent::EndElement {
                name: Some(xml::name::Name {
                    local_name: me.name.as_str(),
                    namespace: None,
                    prefix: None
                })
            };
            w.write(tag_end);
        }
        let doc_begin: WriterXmlEvent = WriterXmlEvent::StartDocument {
            version: XmlVersion::Version10,
            encoding: Some("UTF-8"),
            standalone: None
        };
        writer.write(doc_begin);
        _recursive_write(self, writer);
    }

    pub fn to_path(&self, path: &str) -> anyhow::Result<()> {
        let mut file = File::create(path)?;
        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut file);

        self.to_writer(&mut writer);

        Ok(())
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

impl XmlTag {
    pub fn show_local_tag(&self) {
        static ATOMIC_INDENT: &'static str = "    ";
        let indent = ATOMIC_INDENT;
        println!("∧∧∧∧∧∧∧∧∧∧∧∧∧∧∧∧∧∧∧∧∧");
        println!(
            "{0}+<{1} {2}> {3}\n",
            "", self.name,
            self.attribs.iter()
            .map(|attr: &XmlAttrib| {format!("{}={}", attr.name, attr.value)})
            .fold("".to_string(), |r, s| {r + s.as_str() + " "}),
            {
                if self.value.is_some() {self.value.as_ref().unwrap()}
                else {""}
            }
        );
        for tag in self.children.iter() {
            println!("{0} +{1}", indent, tag.name);
        }
        println!(
            "{0}-</{1}>",
            "",
            self.name
        );
        println!("∨∨∨∨∨∨∨∨∨∨∨∨∨∨∨∨∨∨∨∨∨\n");
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
        println!("{:?}", tree.unwrap());
    }

    #[test]
    fn ed() {
        let tree = XmlTag::from_path("test/template.musicxml").unwrap();
        println!("{:?}", tree);
        tree.to_path("test/outtemplate.xml");
    }
}