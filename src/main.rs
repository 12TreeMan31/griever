use bzip2::read::MultiBzDecoder;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::str;

// Takes the first article and puts any extra data into overflow
fn chip(
    reader: &mut MultiBzDecoder<BufReader<File>>,
    overflow: &mut Vec<u8>,
) -> io::Result<Vec<u8>> {
    let endx: Regex = Regex::new("</page>").unwrap();

    let mut combined: Vec<u8> = overflow.clone();
    let mut leftover: Vec<u8>;
    let mut buf: Vec<u8> = vec![0; 4096];

    loop {
        // First look at the old data and see if it contains </page>

        let regex_str: &str = str::from_utf8(&combined).unwrap_or_else(|error| unsafe {
            // Only convert the valid utf8 to a string
            str::from_utf8_unchecked(&combined[..error.valid_up_to()])
        });

        if let Some(off) = endx.find(regex_str) {
            *overflow = combined[off.end()..combined.len()].to_vec();
            //overflow.extend_from_slice(&combined[off.end()..combined.len() - 1]);
            // Return the buffer ending at the found offset
            return Ok(combined[..off.end()].to_vec());
        } else {
            // Save the end of the current chunk to check against the start of the next chunk
            let leftover_size: usize = "</page>".len().min(combined.len());
            leftover = combined.split_off(combined.len() - leftover_size);
            overflow.append(&mut combined);
        }

        let bytes: usize = reader.read(&mut buf)?;
        if bytes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "End of file reached",
            ));
        }

        combined = leftover.clone();
        combined.extend_from_slice(&buf[..bytes]);
    }
}

fn main() -> io::Result<()> {
    let f: File =
        File::open("/home/treeman/Downloads/enwiki-20231220-pages-articles-multistream.xml.bz2")?;
    // NEVER access the internal reader as it will break the decoder; instead read and save the data somewhere else
    let reader: BufReader<File> = BufReader::new(f);
    let mut reader: MultiBzDecoder<BufReader<File>> = MultiBzDecoder::new(reader);

    // Alignment
    let mut buf: Vec<u8> = vec![0; 4069];
    reader.read(&mut buf)?;

    let mut buf: Vec<u8>;
    let mut overflow: Vec<u8> = Vec::new();
    let mut i: u64 = 0;

    loop {
        buf = chip(&mut reader, &mut overflow)?;
        i += 1;
        println!("{i}");
    }
}
