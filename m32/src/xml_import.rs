use std::path::Path;
use anyhow::{anyhow, Context};
use crate::score::*;
use adaxml::tag::*;
use adaxml::io::*;
use adaxml::iter::*;
use crate::attribs::{BeatDivision, ClefType, KeySignature, TimeSig};
use crate::part::MeasuredPart;

pub struct PartAttributes {
    division: BeatDivision,
    key_fifths: KeySignature,
    time_sig: TimeSig,
    clef_signs: Vec<ClefType>,
    staves: u8
}

pub fn measured_score_from_path(path: &str) -> anyhow::Result<MeasuredScore>
{
    let tag = XmlTag::from_path(path).context("Can't open xml from path")?;
    println!("{:?}", tag);
    measured_score_from_tag(&tag)
}

pub fn measured_score_from_tag(tag: &XmlTag) -> anyhow::Result<MeasuredScore> {
    let mut mscore = MeasuredScore::new("".to_string());
    for master_tag in tag.children {
        if master_tag.name != "score-partwise" {
            return Err(anyhow!("<score-partwise> tag not found"));
        }

        // extract work-title
        let work_title
            = master_tag
            .get_child_with_name("work")
            .get_child_with_name("work-title").ok_or("Can't find <work-title>")?
            .value
            .clone();
        if work_title.is_some() { mscore.title = work_title.unwrap() }

        // find part-list
        let part_list_tag
            = master_tag.get_child_with_name("part-list")
            .ok_or(anyhow!("<part-list> not found"))?;

        // parse part-list
        let xml_part_headers = part_list_tag.all_child_with_name("score-part");

        todo!()
    }
    Ok (mscore)
}

pub fn measured_parts_from_tag(tag: &XmlTag, part_name: String)
    -> anyhow::Result<Vec<MeasuredPart>>
{
    // look for <attributes> before parsing
    let attr_tag
        = tag
        .get_desc_with_name("attributes")
        .ok_or(anyhow!("<attributes> for part not found"))?;

}

pub fn part_attributes_from_tag(tag: &XmlTag)
    -> anyhow::Result<PartAttributes>
{
    let mut part_attrs = PartAttributes {
        division: 0,
        key_fifths: 0,
        time_sig: TimeSig::from_integer(0),
        clef_signs: vec![],
        staves: 0
    };
    part_attrs.division
        = tag
        .get_child_with_name("divisions").context("Cannot find <division> tag")?
        .value.context("<division> tag contains no value")?
        .parse().context("Can't parse value in <division>")?;

    {
        let fifths = tag
            .get_child_with_name("key")?
            .get_child_with_name("fifths")?
            .value
            .clone();
        part_attrs.key_fifths = match fifths {
            Some(t) => t.parse().context("Can't parse value of <fifths>")?,
            None => 0
        };
    }

    {
        let time_tag = tag
            .get_child_with_name("time").context("Can't find <time>")?;
        part_attrs.time_sig = TimeSig::from(
            time_tag.get_child_value_as("beats"),
            time_tag.get_child_value_as("beat-type")
        );
    }

    {
        let staves = tag.get_child_value_as::<u8>("staves");
        part_attrs.staves = staves.unwrap_or(1);
    }

    {
        let clefs = tag.all_child_with_name("clef").collect();
        if clefs.empty() { return Err(anyhow!("<clef> not found")) };
        part_attrs
        .clef_signs
        .extend(
            clefs
            .iter()
            .map(|c| {})
        )
    }

    Ok(part_attrs)
}

#[cfg(test)]
mod tests {
    use crate::xml_import::measured_score_from_path;

    #[test]
    fn test () {
        let m = measured_score_from_path("test/template.musicxml").unwrap();
    }
}