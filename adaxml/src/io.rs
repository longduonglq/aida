use super::tag::*;
use xml::reader::*;

impl XmlTag
{
    fn from_path(path: &str) {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use xml::EventReader;

    #[test]
    fn exp(){
        let mut reader = EventReader::from_str("    <score-part id=\"P1\">
      <part-name>Soprano</part-name>
      <part-abbreviation>S.</part-abbreviation>
      <score-instrument id=\"P1-I1\">
        <instrument-name>Soprano</instrument-name>
        </score-instrument>
      <midi-device id=\"P1-I1\" port=\"1\"></midi-device>
      <midi-instrument id=\"P1-I1\">
        <midi-channel>1</midi-channel>
        <midi-program>1</midi-program>
        <volume>78.7402</volume>
        <pan>0</pan>
        </midi-instrument>
      </score-part>");

        for x in reader {
            println!("{:?}", x);
        }
    }
}