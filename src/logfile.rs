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
use std::path::Path;
use std::io;

pub struct LogFile {
    f: File,
    start: u64,
    last_sync_time: u64,
}

impl LogFile {
    pub fn new(path: &Path, start: u64) -> Result<LogFile, io::Error> {
        let mut opts = OpenOptions::new();
        opts.write(true).append(true).create(true);
        opts.open(&path.join(logname(start).as_slice())).map(move |f| {
            let mtime = f.metadata().unwrap().modified();
            LogFile {
                f: f,
                start: start,
                last_sync_time: mtime
            }
        })
    }

    pub fn len(&self) -> u64 {
        self.f.metadata().unwrap().len()
    }

    pub fn max_offset(&self) -> u64{
        // this will be optimized when we have indexes
        0
    }
}

fn logname(index: u64) -> String {
    format!("{:016x}.log", index)
}
