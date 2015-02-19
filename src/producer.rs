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
use std::path::Path;
use std::time::duration::Duration;
use std::io::{self, Read, Write, Seek, SeekFrom};

use logfile::LogFile;
use store::WriteStore;
use sync_policy::SyncPolicy;

pub struct ProducerOptions {
    sync_policy: SyncPolicy,
    file_roll_size: u64,
    blocking_minimum_retention: Option<Duration>,
    max_total_bytes: u64,
    max_file_age: Option<Duration>,
}

pub struct Producer<'a> {
    store: WriteStore<'a>,
}

impl Copy for ProducerOptions {}

impl<'l> Producer<'l> {
    pub fn new(directory: &str, options: ProducerOptions) -> Result<Producer, io::Error> {
        let store = try!(WriteStore::new(directory, options));
        Ok(Producer { store: store })
    }

    pub fn new_default(directory: &str) -> Result<Producer, io::Error> {
        let sp = SyncPolicy::Periodic(Duration::seconds(1));
        let opts = ProducerOptions {
            sync_policy: sp,
            file_roll_size: 67_108_864,
            blocking_minimum_retention: None,
            max_total_bytes: 536_870_912,
            max_file_age: None,
        };
        Producer::new(directory, opts)
    }

    pub fn append(&'l mut self, msg: &[u8]) -> Result<(), io::Error> {
        self.store.append(msg)
    }
}
