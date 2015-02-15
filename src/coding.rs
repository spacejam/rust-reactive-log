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
extern crate quickcheck;
use std::num::ToPrimitive;

pub fn encode_u32(frameSize: u32) -> [u8; 4] {
    let mut buf = [0u8; 4];
    buf[0] = (0xff & (frameSize >> 24)).to_u8().unwrap();
    buf[1] = (0xff & (frameSize >> 16)).to_u8().unwrap();
    buf[2] = (0xff & (frameSize >> 8)).to_u8().unwrap();
    buf[3] = (0xff & (frameSize)).to_u8().unwrap();
    buf
}

pub fn decode_u32(buf: [u8; 4]) -> u32 {
    ((buf[0] & 0xff).to_u32().unwrap() << 24) |
    ((buf[1] & 0xff).to_u32().unwrap() << 16) |
    ((buf[2] & 0xff).to_u32().unwrap() << 8)  |
    ((buf[3] & 0xff)).to_u32().unwrap()
}

pub fn encode_u64(frameSize: u64) -> [u8; 8] {
    let mut buf = [0u8; 8];
    buf[0] = (0xff & (frameSize >> 56)).to_u8().unwrap();
    buf[1] = (0xff & (frameSize >> 48)).to_u8().unwrap();
    buf[2] = (0xff & (frameSize >> 40)).to_u8().unwrap();
    buf[3] = (0xff & (frameSize >> 32)).to_u8().unwrap();
    buf[4] = (0xff & (frameSize >> 24)).to_u8().unwrap();
    buf[5] = (0xff & (frameSize >> 16)).to_u8().unwrap();
    buf[6] = (0xff & (frameSize >> 8)).to_u8().unwrap();
    buf[7] = (0xff & (frameSize)).to_u8().unwrap();
    buf
}

pub fn decode_u64(buf: [u8; 8]) -> u64 {
    ((buf[0] & 0xff).to_u64().unwrap() << 56) |
    ((buf[1] & 0xff).to_u64().unwrap() << 48) |
    ((buf[2] & 0xff).to_u64().unwrap() << 40) |
    ((buf[3] & 0xff).to_u64().unwrap() << 32) |
    ((buf[4] & 0xff).to_u64().unwrap() << 24) |
    ((buf[5] & 0xff).to_u64().unwrap() << 16) |
    ((buf[6] & 0xff).to_u64().unwrap() << 8)  |
    ((buf[7] & 0xff)).to_u64().unwrap()
}

#[test]
fn equiv_u32() {
    use self::quickcheck::quickcheck;

    fn prop(xs: Vec<u32>) -> bool {
        for x in xs {
            if x != decode_u32(encode_u32(x)) {
                return false
            }
        }
        true
    }
    quickcheck(prop as fn(Vec<u32>) -> bool);
}

#[test]
fn equiv_u64() {
    use self::quickcheck::quickcheck;

    fn prop(xs: Vec<u64>) -> bool {
        for x in xs {
            if x != decode_u64(encode_u64(x)) {
                return false
            }
        }
        true
    }
    quickcheck(prop as fn(Vec<u64>) -> bool);
}
