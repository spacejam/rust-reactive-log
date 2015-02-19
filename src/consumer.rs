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
use std::num::{self, ToPrimitive};

use coding::{decode_u32, decode_u64};
use store::ReadStore;
use message_and_offset::MessageAndOffset;
use logfile::LogFile;

pub trait Consumer<'a> {
    pub fn read<'b>(&mut self) -> Option<MessageAndOffset<'b>>;
}

pub struct BasicConsumer<'a> {
    store: ReadStore,
}

pub enum Whence {
    Latest,
    Oldest,
    Position(u64),
}

pub enum ConsumerStyle<'a> {
    ClientTxConsumer(&'a str),
    GlobalTxConsumer,
}

impl Copy for Whence {}

impl<'a> BasicConsumer<'a> {
    pub fn new(directory: &str, whence: Whence) -> Result<BasicConsumer, io::Error> {
        ReadStore::new(directory, whence).map(|rs| {
            try!(rs.seek(whence));
            BasicConsumer { store: rs }
        }
    }
}

impl<'a> Consumer for BasicConsumer<'a> {
    pub fn read<'b>(&mut self) -> Option<MessageAndOffset<'b>> {
        self.store.read()
    }
}

impl<'a,'c> Iterator for BasicConsumer<'c> {
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
