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

use logfile::LogFile;

pub trait Store {
    pub fn active_log(&self) -> Option<LogFile>;
    pub fn log_for_index(&self, u64) -> Option<LogFile>;
}

pub struct WriteStore<'a> {
    active_log: Option<LogFile>,
    stores: BTreeMap<u64, LogFile<'a>>,
    sync_policy: SyncPolicy,
}

pub struct ReadStore<'a> {
    active_log: Option<LogFile>,
    stores: BTreeMap<u64, LogFile<'a>>,
}

impl<'a> WriteStore<'a> {
    pub fn new<'a>(directory: &'a str, sync_policy: SyncPolicy) -> Result<WriteStore<'a>> {
        let mut opts = OpenOptions::new();
        opts.write(true).append(true).create(true);
        let path = Path::new(directory);
        let stores = stores_from_dir(path);

        if stores.len() == 0 {
            println!("found no suitable log files, initializing new one.");
            let initial_log_file = try!(LogFile::new(directory, 0));
            stores.insert(0, initial_log_file);
        }

        let leading_file = stores.range(Unbounded, Unbounded)
                                 .next_back().unwrap().1;
        WriteStore {
            active_log: leading_file,
            stores: stores,
            sync_policy: sync_policy,
        }
    }

    pub fn append(&'l mut self, msg: &[u8]) -> Result<(), io::Error> {
        self.max_offset += 1;
        if self.should_roll() {
            self.roll_active_file();
        }
        let mut active_file = self.active_log_file().unwrap();
        active_file.append(self.max_offset, msg)
    }

    fn should_roll(&'l self) -> bool {
        let af = self.active_log_file().unwrap();
        let should = af.len() > self.options.file_roll_size;
        println!("current len: {} max len: {} should_roll: {}", af.len(),
        self.options.file_roll_size, should);
        should
    }

    fn active_log_file(&'l self) -> Option<&'l LogFile> {
        self.stores.range(Unbounded, Unbounded)
                   .next_back()
                   .map(move |index_log_file| { index_log_file.1 })
    }

    fn roll_active_file(&self) {
        //TODO get max index, create new file, add to stores map
    }
}

impl<'a> ReadStore<'a> {
    pub fn new<'a>(directory: &'a str) -> Result<ReadStore<'a>> {
        let mut opts = OpenOptions::new();
        opts.read(true);
        let path = Path::new(directory);

        ReadStore {
            active_log: None,
            stores: try!(stores_for_dir(directory)),
        }
    }

    pub fn read<'b>(&mut self) -> Option<MessageAndOffset<'b>> {
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

    fn log_for_index(&'l self, index: u64) -> Option<&'l LogFile> {
        self.stores.range(Unbounded, Included(&index))
                   .next_back()
                   .map(move |index_log_file| { index_log_file.1 })
    }

    pub fn seek(whence: Whence) -> Result<u64, io::Error> {
        //TODO implement
        Ok(0)
    }
}

fn stores_for_dir(directory: &str, opts: OpenOptions) -> Result<BTreeMap<&str, LogFile>, io::Error> {
    if !directory.is_dir() {
        println!("attempting to create new log directory: {}",
                 directory.display());
        try!(fs::create_dir_all(&directory));
    }

    let mut stores: BTreeMap<u64, LogFile<'l>> = BTreeMap::new();

    for possible_file in try!(read_dir(directory)) {
        let f = try!(possible_file);
        let fpath = f.path();
        let fname = fpath.file_name().unwrap().to_str().unwrap();
        let splits = fname.split_str(".").collect::<Vec<&str>>();
        if splits.len() == 2 && splits[1] == "log" {
            num::from_str_radix::<u64>(splits[0], 16).map(|number| {
                println!("found log file: {}", number);
                let log_file = try!(LogFile::new(directory, number));
                stores.insert(number, log_file);
            });
        }
    }
    stores
}
