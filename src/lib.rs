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
#![crate_id = "reactive_log"]
#![crate_type = "lib"]
#![allow(dead_code)]

pub use log::{
    Log,
    Options,
    SyncPolicy,
};

use logfile::LogFile;
use coding::{
    encode_u32,
    decode_u32,
    encode_u64,
    decode_u64,
};
pub mod log;
mod logfile;
mod coding;

#[test]
fn write() {
    use std::path::Path;
    let mut log = Log::new_default(Path::new("/tmp/bananaz/bad/diddety/")).unwrap();
    log.write(b"hello world");
    assert!(log.read().unwrap() == b"hello world");
}
