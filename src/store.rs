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
use std::{self, num};
use std::io::{self, Read, Write, Seek, SeekFrom, BufWriter};
use std::fs::{self, File, OpenOptions, read_dir, PathExt};
//use std::old_io::fs::PathExtensions;

use coding::{decode_u32, decode_u64};
use whence::Whence;
use logfile::LogFile;
use sync_policy::SyncPolicy;
use producer::ProducerOptions;
use message_and_offset::MessageAndOffset;

pub trait Store {
    pub fn active_log(&self) -> Option<LogFile>;
    pub fn log_for_index(&self, u64) -> Option<LogFile>;
}

pub struct WriteStore<'a> {
    active_log: Option<&'a LogFile<'a>>,
    max_offset: u64,
    stores: BTreeMap<u64, LogFile<'a>>,
    options: ProducerOptions,
}

pub struct ReadStore<'a> {
    active_log: Option<LogFile<'a>>,
    stores: BTreeMap<u64, LogFile<'a>>,
}

impl<'a> WriteStore<'a> {
    pub fn new<'b>(directory: &'b str, options: ProducerOptions) -> Result<WriteStore<'b>, io::Error> {
        let mut opts = OpenOptions::new();
        opts.write(true).append(true).create(true);
        let path = Path::new(directory);
        let stores = try!(stores_from_dir(directory, opts));

        if stores.len() == 0 {
            println!("found no suitable log files, initializing new one.");
            let zero_path = Path::new(format!("{:016x}.log", 0));
            let initial_log_file = try!(LogFile::new(&zero_path, opts));
            stores.insert(0, initial_log_file);
        }

        let leading_file = stores.range(Unbounded, Unbounded)
                                 .next_back().unwrap().1;
        Ok(WriteStore {
            active_log: Some(leading_file),
            max_offset: leading_file.max_offset(),
            stores: stores,
            options: options,
        })
    }

    pub fn append(&'a mut self, msg: &[u8]) -> Result<(), io::Error> {
        self.max_offset += 1;
        if self.should_roll() {
            self.roll_active_file();
        }
        self.active_log.unwrap().append(self.max_offset, msg)
    }

    //TODO do this check before writing, so that the limit is only crossed
    //     when a single massive message exceeds the threshold and gets its
    //     own file.
    fn should_roll(&'a self) -> bool {
        let af = self.active_log_file().unwrap();
        let should = af.len() > self.options.file_roll_size;
        println!("current len: {} max len: {} should_roll: {}", af.len(),
        self.options.file_roll_size, should);
        should
    }

    fn active_log_file(&'a self) -> Option<&'a LogFile> {
        self.stores.range(Unbounded, Unbounded)
                   .next_back()
                   .map(move |index_log_file| { index_log_file.1 })
    }

    fn roll_active_file(&self) {
        //TODO get max index, create new file, add to stores map
    }
}

impl<'a> ReadStore<'a> {
    //TODO initialize correct file and offset using whence
    pub fn new<'b>(directory: &'b str, whence: Whence) -> Result<ReadStore<'b>, io::Error> {
        let mut opts = OpenOptions::new();
        opts.read(true);
        let path = Path::new(directory);

        Ok(ReadStore {
            active_log: None,
            stores: try!(stores_from_dir(directory, opts)),
        })
    }

    pub fn read<'b>(&mut self) -> Option<MessageAndOffset<'b>> {
        let original_pos = self.active_log.unwrap().f.seek(SeekFrom::Current(0)).unwrap();
        let offset_buf = &mut[0u8; 8];
        let size_buf = &mut[0u8; 4];
        
        // loop acts as "poor man's goto" for streamlined error handling
        //TODO traverse files if we hit the end
        loop {
            if self.active_log.unwrap().f.read(offset_buf).unwrap() < 8 {
                break;
            }
            let msg_offset = decode_u64(*offset_buf);

            if self.active_log.unwrap().f.read(size_buf).unwrap() < 4 {
                break;
            }
            let msg_size = decode_u32(*size_buf);

            let mut msg = Vec::with_capacity(msg_size as usize);
            unsafe { msg.set_len(msg_size as usize); }
            let mut s = msg.as_mut_slice();
            let n = self.active_log.unwrap().f.read(s).unwrap();
            if n < msg_size as usize {
                break;
            }
            return Some(MessageAndOffset {
                message: msg,
                offset: msg_offset
            });
        }
        // if we couldn't read a complete message, seek back
        self.active_log.unwrap().f.seek(SeekFrom::Start(original_pos)).unwrap();
        None
    }

    //TODO handle 0 case where we need to pick the first log available
    fn log_for_index(&'a self, index: u64) -> Option<LogFile<'a>> {
        self.stores.range(Unbounded, Included(&index))
                   .next_back()
                   .map(move |index_log_file| { *index_log_file.1 })
    }

    //TODO test the hell out of this
    pub fn seek(&'a mut self, whence: Whence) -> Result<u64, io::Error> {
        let stop_pos = match whence {
            Whence::Oldest => 0,
            Whence::Latest => std::u64::MAX,
            Whence::Position(p) => p,
        };

        self.active_log = self.log_for_index(stop_pos);

        // this will be optimized when we have indexes
        let original_pos = self.active_log.unwrap().f.seek(SeekFrom::Current(0)).unwrap();
        let file_size = self.active_log.unwrap().f.metadata().unwrap().len();

        let mut msg_offset = None;
        let offset_buf = &mut[0u8; 8];
        let size_buf = &mut[0u8; 4];

        loop {
            if self.active_log.unwrap().f.read(offset_buf).unwrap() < 8 {
                break;
            }

            if self.active_log.unwrap().f.read(size_buf).unwrap() < 4 {
                break;
            }
            let msg_size = decode_u32(*size_buf);

            let old_pos = self.active_log.unwrap().f.seek(SeekFrom::Current(0)).unwrap();
            println!("max_offset: skipping ahead {} bytes", msg_size);
            let new_pos = self.active_log.unwrap().f.seek(SeekFrom::Current(msg_size as i64)).unwrap();
            if (new_pos - old_pos < msg_size as u64) {
                break;
            }
            msg_offset = Some(decode_u64(*offset_buf));
        }

        self.active_log.unwrap().f.seek(SeekFrom::Start(original_pos)).unwrap();
        Ok(msg_offset.unwrap_or(0))
    }
}

fn stores_from_dir<'a>(directory: &str, opts: OpenOptions)
    -> Result<BTreeMap<u64, LogFile<'a>>, io::Error> {
    let dir_path = Path::new(directory);
    if !dir_path.is_dir() {
        println!("attempting to create new log directory: {}",
                 directory);
        try!(fs::create_dir_all(&directory));
    }

    let mut stores: BTreeMap<u64, LogFile<'a>> = BTreeMap::new();

    for possible_file in try!(read_dir(directory)) {
        let f = try!(possible_file);
        let fpath = f.path();
        let fname = fpath.file_name().unwrap().to_str().unwrap();
        let splits = fname.split_str(".").collect::<Vec<&str>>();
        if splits.len() == 2 && splits[1] == "log" {
            num::from_str_radix::<u64>(splits[0], 16).map(|number| {
                println!("found log file: {}", number);
                let log_file = try!(LogFile::new(&fpath, opts));
                Ok(stores.insert(number, log_file))
            });
        }
    }
    Ok(stores)
}
