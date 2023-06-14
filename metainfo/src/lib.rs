#![feature(slice_as_chunks)]

use bencode::{FromBencode, Bencode, NumFromBencodeError, StringFromBencodeError, VecFromBencodeError};
use bencode::util::ByteString;

#[derive(Debug, PartialEq)]
pub struct InfoFiles {
    files: Vec<File>,
    is_private: bool,
    name: String,
    piece_length: u64,
    pieces: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct InfoFile {
    length: u64,
    is_private: bool,
    name: String,
    piece_length: u64,
    pieces: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct File {
    length: u64,
    path: Vec<String>
}

#[derive(Debug, PartialEq)]
pub struct Piece {
    length: u64,
    path: Vec<String>
}

#[derive(PartialEq,  Debug)]
pub struct MetaInfoFiles {
    announce: String,
    created_by: Option<String>,
    creation_date: u64,
    info: InfoFiles,
    comment: Option<String>,

}

#[derive(PartialEq,  Debug)]
pub struct MetaInfoFile {
    announce: String,
    created_by: Option<String>,
    creation_date: u64,
    info: InfoFile,
    comment: Option<String>,
}

#[derive(Debug)]
pub enum ParseMetainfoError {
    DoesntContainAnnounce,
    DoesntContainCreationDate,
    DoesntContainInfo,
    DoesntContainInfoPrivate,
    DoesntContainInfoName,
    DoesntContainInfoPieceLength,
    DoesntContainInfoPieces,
    DoesntContainInfoFiles,
    DoesntContainInfoFileLength,
    DoesntContainInfoFilePath,
    DoesntContainLength,
    NotADict,
    NotANumber(NumFromBencodeError),
    NotAString(StringFromBencodeError),
    NotAStringList(VecFromBencodeError<StringFromBencodeError>),
    NotANumList(VecFromBencodeError<NumFromBencodeError>)
}

impl FromBencode for InfoFiles {
    type Err = ParseMetainfoError;

    fn from_bencode(bencode: &bencode::Bencode) -> Result<InfoFiles, ParseMetainfoError> {
        use ParseMetainfoError::*;

        match bencode {
            &Bencode::Dict(ref m) => {
                let files = match m.get(&ByteString::from_str("files")) {
                    Some(f) => FromBencode::from_bencode(f).map_err(|_| NotADict),
                    _ => Err(DoesntContainInfoFiles)
                }?;

                let is_private: u8 = match m.get(&ByteString::from_str("private")) {
                    Some (i) => FromBencode::from_bencode(i).map_err(NotANumber),
                    _ => Err(DoesntContainInfoPrivate)
                }?;

                let name = match m.get(&ByteString::from_str("name")) {
                    Some (i) => FromBencode::from_bencode(i).map_err(NotAString),
                    _ => Err(DoesntContainInfoName)
                }?;

                let piece_length = match m.get(&ByteString::from_str("piece length")) {
                    Some (i) => FromBencode::from_bencode(i).map_err(NotANumber),
                    _ => Err(DoesntContainInfoPieceLength)
                }?;
                
                println!("Parsing chunks");
                let pieces: Vec<String> = match m.get(&ByteString::from_str("pieces")) {
                    Some (p) => match p {
                        &Bencode::ByteString(ref v) => Ok(parse_pieces(v)),
                        _ => Err(DoesntContainInfoPieces)
                    }
                    _ => Err(DoesntContainInfoPieces)
                }?;

                Ok(InfoFiles { files, is_private: is_private != 0, name, piece_length, pieces })
            },
            _ => Err(NotADict)
        }
    }
}

impl FromBencode for File {
    type Err = ParseMetainfoError;

    fn from_bencode(bencode: &bencode::Bencode) -> Result<File, ParseMetainfoError> {
        use ParseMetainfoError::*;

        match bencode {
            &Bencode::Dict(ref m) => {
                let length = match m.get(&ByteString::from_str("length")) {
                    Some(l) => FromBencode::from_bencode(l).map_err(NotANumber),
                    _ => Err(DoesntContainInfoFileLength)
                }?;

                let path = match m.get(&ByteString::from_str("path")) {
                    // This should assert for empty path list
                    Some(p) => FromBencode::from_bencode(p).map_err(NotAStringList),
                    _ => Err(DoesntContainInfoFilePath)
                }?;

                Ok(File { length, path })
            },
            _ => Err(NotADict)
        }
    }
}

impl FromBencode for MetaInfoFiles {
    type Err = ParseMetainfoError;

    fn from_bencode(bencode: &bencode::Bencode) -> Result<MetaInfoFiles, ParseMetainfoError> {
        use ParseMetainfoError::*;

        match bencode {
            &Bencode::Dict(ref m) => {
                let announce = match m.get(&ByteString::from_str("announce")) {
                    Some (a) => FromBencode::from_bencode(a).map_err(NotAString),
                    _ => Err(DoesntContainAnnounce)
                }?;
                let created_by = match m.get(&ByteString::from_str("created by")) {
                    Some (a) => FromBencode::from_bencode(a).map_err(NotAString),
                    _ => Ok(None)
                }?;
                let creation_date = match m.get(&ByteString::from_str("creation date")) {
                    Some (a) => FromBencode::from_bencode(a).map_err(NotANumber),
                    _ => Err(DoesntContainCreationDate)
                }?;

                let info = match m.get(&ByteString::from_str("info")) {
                    Some (i) => FromBencode::from_bencode(i),
                    _ => Err(DoesntContainInfo)
                }?;

                let comment = match m.get(&ByteString::from_str("comment")) {
                    Some(c) => match FromBencode::from_bencode(c) {
                        Ok(comment) => Ok(Some(comment)),
                        Err(_) => Ok(None)
                    },
                    _ => Ok(None)
                }?;

                Ok(MetaInfoFiles {
                    announce,
                    created_by,
                    creation_date,
                    info,
                    comment
                })
            },
            _ => Err(NotADict)
        }
    }
}

impl FromBencode for MetaInfoFile {
    type Err = ParseMetainfoError;

    fn from_bencode(bencode: &bencode::Bencode) -> Result<MetaInfoFile, ParseMetainfoError> {
        use ParseMetainfoError::*;

        match bencode {
            &Bencode::Dict(ref m) => {
                let announce = match m.get(&ByteString::from_str("announce")) {
                    Some (a) => FromBencode::from_bencode(a).map_err(NotAString),
                    _ => Err(DoesntContainAnnounce)
                }?;
                let created_by = match m.get(&ByteString::from_str("created by")) {
                    Some (a) => FromBencode::from_bencode(a).map_err(NotAString),
                    _ => Ok(None)
                }?;
                let creation_date = match m.get(&ByteString::from_str("creation date")) {
                    Some (a) => FromBencode::from_bencode(a).map_err(NotANumber),
                    _ => Err(DoesntContainCreationDate)
                }?;

                let info = match m.get(&ByteString::from_str("info")) {
                    Some (i) => FromBencode::from_bencode(i),
                    _ => Err(DoesntContainInfo)
                }?;

                let comment = match m.get(&ByteString::from_str("comment")) {
                    Some(c) => match FromBencode::from_bencode(c) {
                        Ok(comment) => Ok(Some(comment)),
                        Err(_) => Ok(None)
                    },
                    _ => Ok(None)
                }?;

                Ok(MetaInfoFile {
                    announce,
                    created_by,
                    creation_date,
                    info,
                    comment
                })
            },
            _ => Err(NotADict)
        }
    }
}

impl FromBencode for InfoFile {
    type Err = ParseMetainfoError;

    fn from_bencode(bencode: &bencode::Bencode) -> Result<InfoFile, ParseMetainfoError> {
        use ParseMetainfoError::*;

        match bencode {
            &Bencode::Dict(ref m) => {
                let length = match m.get(&ByteString::from_str("length")) {
                    Some(l) => FromBencode::from_bencode(l).map_err(NotANumber),
                    _ => Err(DoesntContainLength)
                }?;

                let is_private: u8 = match m.get(&ByteString::from_str("private")) {
                    Some (i) => FromBencode::from_bencode(i).map_err(NotANumber),
                    _ => Err(DoesntContainInfoPrivate)
                }?;

                let name = match m.get(&ByteString::from_str("name")) {
                    Some (i) => FromBencode::from_bencode(i).map_err(NotAString),
                    _ => Err(DoesntContainInfoName)
                }?;

                let piece_length = match m.get(&ByteString::from_str("piece length")) {
                    Some (i) => FromBencode::from_bencode(i).map_err(NotANumber),
                    _ => Err(DoesntContainInfoPieceLength)
                }?;
                
                let pieces: Vec<String> = match m.get(&ByteString::from_str("pieces")) {
                    Some (p) => match p {
                        &Bencode::ByteString(ref v) => Ok(parse_pieces(v)),
                        _ => Err(DoesntContainInfoPieces)
                    }
                    _ => Err(DoesntContainInfoPieces)
                }?;

                Ok(InfoFile { length, is_private: is_private != 0, name, piece_length, pieces })
            },
            _ => Err(NotADict)
        }
    }
}

///
/// Parses the hash pieces from the pieces bytearray in the dict
///
fn parse_pieces(v: &Vec<u8>) -> Vec<String> {
    let (chunks, reminder) = v.as_chunks::<20>();
    
    if reminder.len() > 0 {
        panic!("Invalid pieces chunk");
    }


    let mut hashes: Vec<String> = vec![];

    for c in chunks {
        use core::fmt::Write;

        let mut s = String::with_capacity(c.len() * 2);

        for b in c {
            write!(s, "{:02X}", b).expect("Could not write into the hash buffer string");
        }
        
        hashes.push(s);
    }
    
    hashes
}