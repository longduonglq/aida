use std::iter::{Peekable, zip};
use std::path::Path;
use anyhow::{anyhow, Context};
use fraction::Ratio;
use smallvec::SmallVec;
use adaxml::{drill, drill_helper};
use crate::score::*;
use adaxml::tag::*;
use adaxml::io::*;
use adaxml::iter::*;
use crate::attribs::{BeatDivision, ClefType, Duration, KeySignature, MPInterval, Offset, TimeSig, TimeSigComponent};
use crate::color::Color;
use crate::config::config;
use crate::gnote::Gnote;
use crate::gnote::Gnote::{SimpleNote, Tuplet};
use crate::lyric::Lyric;
use crate::measure::{Measure, measure_length_from_time_sig, MeasureNumberType};
use crate::part::{MeasuredPart, Part};
use crate::pitch::{Alter, DiatonicStep, Octave, Pitch};
use crate::{either_gnote, simple_note, tuplet};
use crate::simple_note::{TieInfo};
use crate::tuplet::NormalNumType;

pub struct PartAttributes {
    division: BeatDivision,
    key_fifths: KeySignature,
    time_sig: TimeSig,
    clef_signs: Vec<ClefType>,
    staves: u8,

    pub measure_length: Duration //computed measure length since used alot
}
/// /////// Part //////// //

pub fn score_from_tag(score_tag: &XmlTag)
    -> anyhow::Result<Score>
{
    let mut score = Score::new("Untitled");

    if score_tag.name != "score-partwise" {
        return Err(anyhow!("<score-partwise> tag not found"));
    }

    // extract work-title
    let work_title
        = score_tag
        .get_child_with_name("work").context("Can't find <work>")?
        .get_child_with_name("work-title").context("Can't find <work-title>")?
        .value
        .clone();
    if work_title.is_some() { score.title = work_title.unwrap_or("Untitled".to_string()) }

    // find part-list
    let part_list_tag
        = score_tag.get_child_with_name("part-list")
        .context("Can't find <part-list>")?;

    // parse part-list
    let xml_part_headers = part_list_tag.all_child_with_name("score-part");
    let number_of_parts = {
        score_tag
            .all_child_with_name("part")
            .map(|part_tag| {
                part_attributes_from_tag(
                    part_tag
                    .get_child_with_name("measure")
                    .and_then(|m| { m.get_child_with_name("attributes")})
                    .ok_or(anyhow!("Can't find <attributes> in <measure>"))
                    .unwrap()
                ).unwrap()
            })
            .map(|attr| { attr.staves })
            .sum::<u8>() as usize
    };
    score.parts.reserve(number_of_parts);

    let xml_parts = score_tag.all_child_with_name("part");
    for (xml_part_header, xml_part) in zip(xml_part_headers, xml_parts) {
        score.parts.push(
            part_from_tag(
                xml_part,
                xml_part_header
                .get_child_value("part-name")
                .map(|c| { c.as_str()})
                .unwrap_or("Untitled part")
            )?
        )
    }
    Ok(score)
}

pub fn part_from_tag(part_tag: &XmlTag, part_name: &str)
    -> anyhow::Result<Part>
{
    // look for <attributes>
    let attrs
        = part_tag
        .get_desc_with_name("attributes")
        .context("Error while parsing <attributes>")
        .and_then(|tag| { part_attributes_from_tag(tag) })
        .context("Can't parse <attributes> to attribute object")?;

    assert_eq!(attrs.clef_signs.len(), 1);
    let mut  part = Part::new(
        part_name.to_string(),
        attrs.key_fifths,
        attrs.clef_signs[0],
        attrs.time_sig
    );

    // parses each notes
    let mut offset_so_far = Offset::from_integer(0);
    for measure_tag in part_tag.all_child_with_name("measure") {
        let mut note_tags = measure_tag.all_child_with_name("note");
        let mut gnotes: Vec<_>
            = xml_notes_to_gnotes(&mut note_tags.peekable(), attrs.division, &attrs)?
            .into_iter()
            .map(move |mut gn| {
                either_gnote!(&mut gn, g => g.interval.displace_start_keep_length(offset_so_far));
                gn
            })
            .collect();
        offset_so_far +=
            gnotes
            .iter()
            .map(|n| { either_gnote!(n, g => g.interval.length) })
            .sum::<Duration>();
        part.gnotes.append(&mut gnotes);
    }

    Ok(part)
}

/// /////// Measured Part //////// ///

pub fn measured_score_from_path(path: &str) -> anyhow::Result<MeasuredScore>
{
    let tag = XmlTag::from_path(path).context("Can't open xml from path")?;
    // println!("{:?}", tag);
    measured_score_from_tag(&tag)
}

pub fn measured_score_from_tag(score_tag: &XmlTag) -> anyhow::Result<MeasuredScore> {
    let mut mscore = MeasuredScore::new("".to_string());

    if score_tag.name != "score-partwise" {
        return Err(anyhow!("<score-partwise> tag not found"));
    }

    // extract work-title
    let work_title
        = score_tag
        .get_child_with_name("work").context("Can't find <work>")?
        .get_child_with_name("work-title").context("Can't find <work-title>")?
        .value
        .clone();
    if work_title.is_some() { mscore.title = work_title.unwrap_or("Untitled".to_string()) }

    // find part-list
    let part_list_tag
        = score_tag.get_child_with_name("part-list")
        .context("Can't find <part-list>")?;

    // parse part-list
    let xml_part_headers = part_list_tag.all_child_with_name("score-part");
    let number_of_parts = {
        score_tag
        .all_child_with_name("part")
        .map(|part_tag| {
            part_attributes_from_tag(
                part_tag
                .get_child_with_name("measure")
                .and_then(|m| { m.get_child_with_name("attributes")})
                .ok_or(anyhow!("Can't find <attributes> in <measure>"))
                .unwrap()
            ).unwrap()
        })
        .map(|attr| { attr.staves })
        .sum::<u8>() as usize
    };
    mscore.measured_parts.reserve(number_of_parts);

    let xml_parts = score_tag.all_child_with_name("part");
    for (xml_part_header, xml_part) in zip(xml_part_headers, xml_parts) {
        mscore.measured_parts.extend(
            measured_parts_from_tag(
                xml_part,
                xml_part_header
                    .get_child_value("part-name")
                    .map(|c| { c.as_str()})
                    .unwrap_or("Untitled part")
            )?
            .into_iter()
        )
    }
    Ok (mscore)
}

pub fn measured_parts_from_tag(part_tag: &XmlTag, part_name: &str)
    -> anyhow::Result<Vec<MeasuredPart>> // returns Vec because eg: piano parts has 2 staves
{
    // look for <attributes> before parsing
    let attrs
        = part_tag
        .get_desc_with_name("attributes")
        .context("Error while parsing <attributes>")
        .and_then(|tag| { part_attributes_from_tag(tag) })
        .context("Can't parse <attributes> to attribute object")?;

    let mut measured_parts: Vec<MeasuredPart> = Vec::new();
    measured_parts.reserve(attrs.staves as usize);

    let number_of_measures
        = part_tag
        .all_desc_with_name("measure")
        .last()
        .map(|t| {
                t.get_attrib_value_as::<usize>("measure")
                .unwrap_or(1)
            }
        )
        .unwrap_or(1);

    measured_parts
    .extend(
        attrs.clef_signs
        .iter()
        .map(|clef_sign| {
            MeasuredPart::new(
                part_name.to_string(),
                attrs.key_fifths,
                clef_sign.clone(),
                attrs.time_sig
            )
        })
        .map(|mut mpart| { mpart.measures.reserve(number_of_measures); mpart})
    );

    // split by <backup>
    for measure_tag in part_tag.all_child_with_name("measure")
    {
        assert!(measured_parts.iter()
            .all(|mea| {
                mea.measures.len() == measured_parts[0].measures.len()
            }));

        let mut cur_stave_number = 1;
        let layers: Vec<_>
            = measure_tag.children.as_slice()
            .split_inclusive(|t| { t.name == "backup"} )
            .collect();

        if layers.len() != attrs.staves as usize {
            return Err(anyhow!("Expected {:?} staves but found {:?} instead", attrs.staves, layers.len()));
        }

        for (layer, mpart) in zip(&layers, &mut measured_parts) {
            let gnote_stream = xml_notes_to_gnotes(
                &mut layer.iter().peekable(),
                attrs.division,
                &attrs)?;

            let cur_measure = mpart.append_empty_measure();
            cur_measure
                .gnotes
                .extend(gnote_stream);
        }
    }

    Ok(measured_parts)
}

/// Intra-measure translation !
// This fn stops when encountering <backup>, which should coincide with end of iterator
fn xml_notes_to_gnotes<'a>(
    gn_tags: &mut Peekable<impl Iterator<Item=&'a XmlTag>>,
    divisions: BeatDivision,
    attrs: &PartAttributes
) -> anyhow::Result<Vec<Gnote>>
{
    let mut gnote_stream = Vec::new();
    while gn_tags.peek().is_some() {
        let tag_name = gn_tags.peek().unwrap().name.as_str();
        match tag_name {
            "note" => {
                let last_gnote_end
                    = gnote_stream
                    .last()
                    .map(|g| { either_gnote!(g, gn => gn.interval.end) } )
                    .unwrap_or(Offset::from_integer(0));

                gnote_stream.push(
                    gnote_from_tag(gn_tags, divisions)
                    .map(|mut g| {
                        either_gnote!(&mut g, gn => gn.interval.displace_start_keep_length(last_gnote_end));
                        g
                    })?
                );
            },
            "backup" => {
                let backup_duration = Duration::new(
                    gn_tags.peek().unwrap().get_child_value_as("duration").unwrap(),
                    divisions
                );
                if backup_duration !=
                Ratio::<BeatDivision>::new(
                    (*attrs.time_sig.numer() as BeatDivision) * BeatDivision::from(4),
                    *attrs.time_sig.denom() as BeatDivision
                )
                {
                    return Err(anyhow!("<backup> behaves in unexpected ways"));
                }
            },
            _ => { return Err(anyhow!("Found tags other than <note>, <backup> in stream")) }
        }
    }
    Ok(gnote_stream)
}

pub fn gnote_from_tag<'a>(gn_tags: &mut Peekable<impl Iterator<Item=&'a XmlTag>>, divisions: BeatDivision)
    -> anyhow::Result<Gnote>
{
    // Some software include tuplet[type='start'] as a tuplet starter signal.
    // Some doesn't do this and we must rely on the fact that <time-mod> is present
    if let Some(cur_gn_tag) = gn_tags.peek() {
        let is_tuplet_start
            = cur_gn_tag.get_child_with_name("tuplet")
                .and_then(|tup| { tup.get_attrib_value("type") })
                .filter(|ty| { ty == &"start"})
                .is_some()
            || cur_gn_tag.does_child_exists("time-modification");

        if is_tuplet_start {
            let time_mod_tag
                = cur_gn_tag.get_child_with_name("time-modification")
                .expect("Apparently tuplet has no <time-modification>");

            let mut simple_notes = Vec::new();
            simple_notes.reserve(time_mod_tag.get_child_value_as::<NormalNumType>("actual-notes").unwrap() as usize);

            let mut offset_so_far = Offset::from_integer(0);
            loop {
                // check endOfTuple before parsing it bc Chord only has one <tuple type=stop> for the first note
                let reached_end_of_tuplet
                    = gn_tags.peek().ok_or(anyhow!("Tag unexpected popped somewhere above"))?
                    .get_child_with_name("tuplet")
                    .and_then(|tup| { tup.get_attrib_value("type") })
                    .filter(|ty| {ty == &"stop"})
                    .is_some();

                let mut tup_member = simple_note_from_tag(gn_tags, divisions)?;
                tup_member.interval.displace_start_keep_length(offset_so_far);
                offset_so_far += tup_member.interval.length;
                simple_notes.push(tup_member);

                if gn_tags.peek().is_none() || reached_end_of_tuplet {
                    break;
                }
            }

            let bare_tup = tuplet::Tuplet::new(
                time_mod_tag.get_child_value_as("normal-notes").unwrap(),
                time_mod_tag.get_child_value_as("actual-notes").unwrap(),
                MPInterval::from_start_and_length(Offset::from_integer(0), offset_so_far),
                simple_notes
            );
            Ok(Gnote::Tuplet(bare_tup))
        } else {
            simple_note_from_tag(gn_tags, divisions)
            .map(|sn| { Gnote::SimpleNote(sn) })
        }
    }
    else {
        Err(anyhow!("Empty iterator"))
    }
}

pub fn simple_note_from_tag<'a>(sn_tag: &mut Peekable<impl Iterator<Item=&'a XmlTag>>, divisions: BeatDivision)
    -> anyhow::Result<simple_note::SimpleNote>
{
    if sn_tag.peek().is_none() {
        return Err(anyhow!("Empty iterator"));
    }

    let mut simple_note = {
        let cur_tag = sn_tag.peek().unwrap();
        // parse duration
        let duration_ticks
            = cur_tag.get_child_value("duration")
            .ok_or(anyhow!("No <duration>"))?
            .parse::<BeatDivision>()
            .context("Can't parse <duration> value")?;
        let duration = Duration::new(duration_ticks, divisions);

        // parse lyrics
        let lyrics
            = lyrics_from_tags(cur_tag.all_child_with_name("lyric"))?;

        // color
        let color: Option<Color>
            = cur_tag.get_attrib_value("color")
            .or(cur_tag.get_desc_with_name("notehead")
            .and_then(|nh| {nh.get_attrib_value("color")}))
            .map(|s| { Color::from_hex_rgb(s)} );

        // encode tie info
        let tie_info
            = tie_info_from_tag(cur_tag.all_desc_with_name("tie"))?;

        let mut simple_note = simple_note::SimpleNote::new(
            Offset::from_integer(0),
            duration,
            lyrics,
            color,
            tie_info
        );

        if cur_tag.does_child_exists("rest") {
            return Ok(simple_note);
        }

        simple_note
    };

    loop {
        let cur_xml_tag = sn_tag.peek().unwrap();
        let pitch_tag
            = cur_xml_tag
            .get_child_with_name("pitch")
            .expect("Not rest yet no <pitch> found");
        let pitch = pitch_from_tag(pitch_tag)?;
        simple_note.pitches.insert(pitch);

        // if chord tone has tie property, propagate to chord
        let potential_tie_info
            = tie_info_from_tag(cur_xml_tag.all_desc_with_name("tie"))?;
        simple_note.tie_info |= potential_tie_info;

        sn_tag.next();
        if sn_tag.peek().is_none()
        || !sn_tag.peek().unwrap().does_child_exists("chord") { break; }
    }

    Ok(simple_note)
}

pub fn tie_info_from_tag<'a>(tie_tag: impl Iterator<Item=&'a XmlTag>)
    -> anyhow::Result<TieInfo>
{
    let xml_ties: Vec<_> = tie_tag.collect();
    if xml_ties.is_empty() { Ok(TieInfo::TieNeither) }
    else {
        let mut tie_info = TieInfo::TieNeither;
        for xml_tie in xml_ties {
            match xml_tie.get_attrib_value("type").unwrap() {
                "start" => { tie_info |= TieInfo::TieStart },
                "stop" => { tie_info |= TieInfo::TieEnd},
                _ => { panic!("Unknown <tie>'s type")}
            }
        }
        Ok(tie_info)
    }
}

pub fn pitch_from_tag(pitch_tag: &XmlTag)
    -> anyhow::Result<Pitch>
{
    let alter: Alter
        = pitch_tag.get_child_value_as::<i32>("alter")
        .unwrap_or(0)
        .into();

    Ok (
        Pitch::new(
            DiatonicStep::from(
                pitch_tag.get_child_value("step")
                .context("Can't parse <step> in <pitch>")?
                .as_str()
            ),
            pitch_tag.get_child_value_as("octave"),
            alter
        )
    )
}

pub fn lyrics_from_tags<'a>(lyric_tags: impl Iterator<Item=&'a XmlTag>)
    -> anyhow::Result<Vec<Lyric>>
{
    let mut lyrics = Vec::new();
    lyrics.reserve(config::EXP_LYRIC_NUM);
    for lyric_tag in lyric_tags {
        lyrics.push(
            Lyric::new(
                lyric_tag.get_attrib_value_as("number")
                    .ok_or(anyhow!("Can't get lyric number"))?,
                lyric_tag.get_child_value("text")
                    .ok_or(anyhow!("Can't extract lyric text"))?
                    .clone()
            )
        )
    }
    Ok(lyrics)
}

pub fn part_attributes_from_tag(tag: &XmlTag)
    -> anyhow::Result<PartAttributes>
{
    let mut part_attrs = PartAttributes {
        division: 0,
        key_fifths: 0,
        time_sig: TimeSig::from_integer(0),
        clef_signs: vec![],
        staves: 0,
        measure_length: Duration::from(0)
    };
    part_attrs.division
        = tag
        .get_child_with_name("divisions").context("Cannot find <division> tag")?
        .value.as_ref().context("<division> tag contains no value")?
        .parse().context("Can't parse value in <division>")?;

    {
        trace_macros!(true);
        // TODO: USE DRILL! macro for easier to read code
        let fifths_tag = drill!(tag; ["key", "fifths", "fs"]);
        part_attrs.key_fifths = tag
            .get_child_with_name("key")
            .and_then( |c| { c.get_child_with_name("fifths") })
            .and_then(|c| { c.value.clone() })
            .map(|s| { s.parse().unwrap_or(0) })
            .unwrap_or(0);
    }

    {
        let time_tag = tag
            .get_child_with_name("time").context("Can't find <time>")?;
        part_attrs.time_sig = TimeSig::from((
            time_tag.get_child_value_as("beats").context("Can't parse <beats>")?,
            time_tag.get_child_value_as("beat-type").context("Can't parse <beat-type>")?
        ));
    }

    {
        let staves = tag.get_child_value_as::<u8>("staves");
        part_attrs.staves = staves.unwrap_or(1);
    }

    {
        let clefs: Vec<_> = tag.all_child_with_name("clef").collect();
        if clefs.is_empty() { return Err(anyhow!("<clef> not found")) };
        part_attrs
        .clef_signs
        .extend(
            clefs
            .iter()
            .map(|tag| {
                tag.get_child_value_as::<ClefType>("sign").context("Can;t parse <clef>::sign").unwrap()
            })
        );
    }

    part_attrs.measure_length = measure_length_from_time_sig(part_attrs.time_sig);

    Ok(part_attrs)
}

// Submeasure (notes, ties, rests, ...) elements

#[cfg(test)]
mod tests {
    use crate::xml_import::measured_score_from_path;

    #[test]
    fn test () {
        let m = measured_score_from_path("test/template.musicxml").unwrap();
    }
}