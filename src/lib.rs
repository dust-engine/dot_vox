use std::fs::File;
use std::io::Read;
use nom::IResult::Done;

#[macro_use]
extern crate nom;

const MAGIC_NUMBER: &'static str = "VOX ";

#[derive(Debug, PartialEq)]
pub struct DotVoxData {
    version: Vec<u8>
}

named!(parse_vox_file <&[u8], DotVoxData>, chain!(
  tag!(MAGIC_NUMBER) ~
  version: take!(4)
  , || DotVoxData { version: version.to_vec() })
);

pub fn load(filename: &str) -> Result<DotVoxData, &'static str> {
  match File::open(filename) {
    Ok(mut f) => {
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer).expect("Unable to read file");
      match parse_vox_file(&buffer) {
        Done(_, parsed) => Ok(parsed),
        _ => Err("Not a valid MagicaVoxel .vox file")
      }
    },
    Err(_) => Err("Unable to load file")
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_file_is_read_successfully() {
        let result = load("resources/placeholder.vox");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DotVoxData { version: vec!(150, 0, 0, 0) });
    }

    #[test]
    fn not_present_file_causes_error() {
        let result = load("resources/not_here.vox");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unable to load file");
    }

    #[test]
    fn non_vox_file_causes_error() {
        let result = load("resources/not_a.vox");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not a valid MagicaVoxel .vox file");
    }
}
