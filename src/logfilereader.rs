/*
   Copyright 2015 Tyler Neely

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/
use std::fs::{self, File, OpenOptions, read_dir, PathExt};
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::num::ToPrimitive;

use coding::{decode_u32, decode_u64};
use messageandoffset::MessageAndOffset;

pub struct LogFileReader {
    lf: File,
    last_pos: u64,
}

impl LogFileReader {
    pub fn new(directory: &Path, start: u64) -> Result<LogFileReader, io::Error> {
        let mut opts = OpenOptions::new();
        opts.read(true);
        let logname = format!("{:016x}.log", start);
        opts.open(&directory.join(logname.as_slice())).map(move |f| {
            LogFileReader {
                lf: f,
                last_pos: 0,
            }
        })
    }

    pub fn read<'a>(&mut self) -> Option<MessageAndOffset<'a>> {
        let original_pos = self.lf.seek(SeekFrom::Current(0)).unwrap();
        let offset_buf = &mut[0u8; 8];
        let size_buf = &mut[0u8; 4];
        
        // loop acts as "poor man's goto" for streamlined error handling
        loop {
            if self.lf.read(offset_buf).unwrap() < 8 {
                break;
            }
            let msg_offset = decode_u64(*offset_buf);

            if self.lf.read(size_buf).unwrap() < 4 {
                break;
            }
            let msg_size = decode_u32(*size_buf);

            let mut msg = Vec::with_capacity(msg_size as usize);
            unsafe { msg.set_len(msg_size as usize); }
            let mut s = msg.as_mut_slice();
            let n = self.lf.read(s).unwrap();
            if n < msg_size as usize {
                break;
            }
            return Some(MessageAndOffset {
                message: msg,
                offset: msg_offset
            });
        }
        // if we couldn't read a complete message, seek back
        self.lf.seek(SeekFrom::Start(original_pos)).unwrap();
        None
    }

    //TODO test the hell out of this
    pub fn max_offset(&mut self) -> u64 {
        // this will be optimized when we have indexes
        let original_pos = self.lf.seek(SeekFrom::Current(0)).unwrap();
        let file_size = self.lf.metadata().unwrap().len();

        let mut msg_offset = None;
        let offset_buf = &mut[0u8; 8];
        let size_buf = &mut[0u8; 4];

        loop {
            if self.lf.read(offset_buf).unwrap() < 8 {
                break;
            }

            if self.lf.read(size_buf).unwrap() < 4 {
                break;
            }
            let msg_size = decode_u32(*size_buf);

            let old_pos = self.lf.seek(SeekFrom::Current(0)).unwrap();
            println!("max_offset: skipping ahead {} bytes", msg_size);
            let new_pos = self.lf.seek(SeekFrom::Current(msg_size as i64)).unwrap();
            if (new_pos - old_pos < msg_size as u64) {
                break;
            }
            msg_offset = Some(decode_u64(*offset_buf));
        }

        self.lf.seek(SeekFrom::Start(original_pos)).unwrap();
        msg_offset.unwrap_or(0)
    }
}

impl<'a> Iterator for LogFileReader {
    type Item = &'a [u8];
    
    #[inline]
    fn next(&mut self) -> Option<&'a [u8]> {
        self.last_pos = self.lf.seek(SeekFrom::Current(0)).unwrap();
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let file_size = self.lf.metadata().unwrap().len();
        if self.last_pos == file_size {
            (0, Some(0))
        } else {
            (1, None)
        }
    }
}
