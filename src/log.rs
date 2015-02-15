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
use std::collections::BTreeMap;
use std::collections::Bound::{Included, Unbounded};
use std::fs::{self, File, OpenOptions, read_dir, PathExt};
use std::io;
use std::num;
use std::path::Path;
use std::time::duration::Duration;

use logfile::LogFile;
use coding::{encode_u32, decode_u32, encode_u64, decode_u64};

pub enum SyncPolicy {
    Always,
    Never,
    Periodic(Duration),
    PerThreadBufferBytes(u32),
    TotalBufferBytes(u32),
}

pub struct Options {
    sync_policy: SyncPolicy,
    file_roll_size: u64,
    blocking_minimum_retention: Option<Duration>,
    max_total_bytes: u64,
    max_file_age: Option<Duration>,
}

pub struct Log<'l> {
    stores: BTreeMap<u64, LogFile>,
    options: Options,
    max_offset: u64,
}

impl<'l> Log<'l> {
    pub fn new(log_directory: &Path, options: Options) -> Result<Log, io::Error> {
        if !log_directory.is_dir() {
            println!("attempting to create new log directory: {}",
                     log_directory.display());
            try!(fs::create_dir_all(&log_directory));
        }

        let mut stores: BTreeMap<u64, LogFile> = BTreeMap::new();
        for possible_file in try!(read_dir(log_directory)) {
            let f = try!(possible_file);
            let fpath = f.path();
            let fname = fpath.file_name().unwrap().to_str().unwrap();
            let splits = fname.split_str(".").collect::<Vec<&str>>();
            if splits.len() == 2 && splits[1] == "log" {
                let number = num::from_str_radix::<u64>(splits[0], 16).unwrap();
                println!("found log file: {}", number);
                let log_file = try!(LogFile::new(log_directory, number));
                stores.insert(number, log_file);
            }
        }

        if stores.len() == 0 {
            println!("found no suitable log files, initializing new one.");
            let initial_log_file = try!(LogFile::new(log_directory, 0));
            stores.insert(0, initial_log_file);
        }

        let max_offset = max_offset_from_stores(&mut stores).unwrap();

        Ok(Log { stores: stores, options: options, max_offset: max_offset })
    }

    pub fn new_default(log_directory: &Path) -> Result<Log, io::Error> {
        let sp = SyncPolicy::Periodic(Duration::seconds(1));
        let opts = Options {
            sync_policy: sp,
            file_roll_size: 67_108_864,
            blocking_minimum_retention: None,
            max_total_bytes: 536_870_912,
            max_file_age: None,
        };
        Log::new(log_directory, opts)
    }

    pub fn write(&mut self, msg: &[u8]) -> Result<(), io::Error> {
        if self.should_roll() {
            self.roll_active_file();
        }
        self.max_offset += 1;
        Ok(())
    }

    pub fn read<'a>(self) -> Result<&'a [u8], io::Error> {
        Ok("ayo".as_bytes())
    }

    fn log_for_index(&'l self, index: u64) -> Option<&'l LogFile> {
        self.stores.range(Unbounded, Included(&index))
                   .next_back()
                   .map(move |index_log_file| { index_log_file.1 })
    }

    fn active_log_file(&'l self) -> Option<&'l LogFile> {
        self.stores.range(Unbounded, Unbounded)
                   .next_back()
                   .map(move |index_log_file| { index_log_file.1 })
    }

    fn should_roll(&self) -> bool {
        let af = self.active_log_file().unwrap();
        let should = af.len() > self.options.file_roll_size;
        println!("current len: {} max len: {} should_roll: {}", af.len(),
        self.options.file_roll_size, should);
        should
    }

    fn roll_active_file(&self) {
        
    }

}

fn max_offset_from_stores(stores: &mut BTreeMap<u64, LogFile>) -> Option<u64> {
    stores.range(Unbounded, Unbounded)
               .next_back()
               .map(move |index_log_file| { index_log_file.1.max_offset() })
}
