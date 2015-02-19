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
use std::io::{self, Read, Write, Seek, SeekFrom, BufWriter};
use std::num::ToPrimitive;

use coding::{encode_u32, encode_u64};

pub struct LogFile<'a> {
    pub f: File,
    last_sync_time: u64,
}

impl<'a> LogFile<'a> {
    pub fn new(path: &Path, opts: OpenOptions) -> Result<LogFile, io::Error> {
        opts.open(&path).map(move |f| {
            let mtime = f.metadata().unwrap().modified();
            LogFile {
                f: f,
                last_sync_time: mtime
            }
        })
    }

    pub fn len(&self) -> u64 {
        self.f.metadata().unwrap().len()
    }

    pub fn append(&mut self, offset: u64, msg: &[u8]) -> Result<(), io::Error> {
        let offset_bytes = encode_u64(offset);
        let size_bytes = encode_u32(msg.len().to_u32().unwrap());
        self.f.write(&offset_bytes);
        self.f.write(&size_bytes);
        self.f.write(msg);
        if self.should_flush() {
            self.sync_all();
        }
        Ok(())
    }

    pub fn should_flush(&self) -> bool {
        false
    }

    pub fn sync_all(&mut self) {
        self.f.sync_all();
    }

    pub fn max_offset(&mut self) -> u64 {
        0
    }
}
