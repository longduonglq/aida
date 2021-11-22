use std::path::Path;
use anyhow::Context;
use crate::score::*;
use adaxml::tag::*;
use adaxml::io::*;
use adaxml::iter::*;

pub fn measured_score_from_path(path: &Path) -> MeasuredScore
{
    let tag = XmlTag::from_path(path).context("Can't open xml from path")?;
    score_from_xml_tag(tag)
}

pub fn measured_score_from_xml_tag(tag: XmlTag) -> MeasuredScore {
    let mscore = MeasuredScore::new("Untitled".to_string());
}