use std::borrow::Cow;
use std::fs::File;
use std::io::{Bytes, Read};
use std::iter::Take;

fn parse_chunk<'a>(bytes: Take<Bytes<File>>) -> Result<Cow<'a, str>, std::string::FromUtf8Error> {
  String::from_utf8(bytes.map(|b| b.unwrap()).collect()).map(|s| s.into())
}

pub fn load(filename: &str) -> Result<(), &'static str> {
  let MAGIC_NUMBER = "VOX "; // om nom nom signficant whitespace

  match File::open(filename) {
    Ok(f) => {
      let iterator = f.bytes();
      match parse_chunk(iterator.take(4)) {
        Ok(magic_number) => {
          if magic_number == MAGIC_NUMBER {
            Ok(())
          } else {
            Err("Not a valid MagicaVoxel .vox file")
          }
        },
        Err(_) => Err("Unable to parse magic number chunk")
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
        //TODO test values returned
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
