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
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::num::{self, ToPrimitive};

use store::ReadStore;
use message_and_offset::MessageAndOffset;
use logfile::LogFile;
use whence::Whence;

pub trait Consumer {
    pub fn read<'b>(&mut self) -> Option<MessageAndOffset<'b>>;
}

pub struct BasicConsumer<'a> {
    store: ReadStore<'a>,
}

pub enum ConsumerStyle<'a> {
    ClientTxConsumer(&'a str),
    GlobalTxConsumer,
}

impl<'a> BasicConsumer<'a> {
    pub fn new(directory: &str, whence: Whence) -> Result<BasicConsumer, io::Error> {
        let rs = try!(ReadStore::new(directory, whence));
        try!(rs.seek(whence));
        Ok(BasicConsumer { store: rs })
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
        //TODO implement
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        //TODO implement
        (0, None)
    }
}
