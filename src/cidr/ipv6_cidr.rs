use std::net::Ipv6Addr;
use std::mem::transmute;
use std::fmt::{self, Formatter, Display, Debug};

use regex::Regex;
use std::cmp::Ordering;

lazy_static! {
    static ref RE_IPV6_CIDR: Regex = {
        Regex::new(r"^(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))(/((12[0-8])|(1[0-1][0-9])|([1-9][0-9])|[0-9]))?$").unwrap()
    };
}

// TODO: Functions

#[inline]
fn get_mask(bits: u8) -> u128 {
    let mut a = [0u8; 16];

    let l = (bits / 8) as usize;

    for i in 0..l {
        a[i] = 255;
    }

    let d = bits % 8;

    if d > 0 {
        a[l] = 0xFF << (8 - d);
    }

    unsafe {
        transmute(a)
    }
}

#[inline]
fn u128_to_u8_array(uint128: u128) -> [u8; 16] {
    unsafe { transmute(uint128) }
}

#[inline]
fn u128_to_u16_array(uint128: u128) -> [u16; 8] {
    let a = u128_to_u8_array(uint128);

    unsafe { transmute([a[1], a[0], a[3], a[2], a[5], a[4], a[7], a[6], a[9], a[8], a[11], a[10], a[13], a[12], a[15], a[14]]) }
}

#[inline]
fn u8_array_to_u128(uint8_array: [u8; 16]) -> u128 {
    unsafe { transmute(uint8_array) }
}

#[inline]
fn u16_array_to_u128(uint8_array: [u16; 8]) -> u128 {
    let a: [u8; 16] = unsafe { transmute(uint8_array) };

    unsafe { transmute([a[1], a[0], a[3], a[2], a[5], a[4], a[7], a[6], a[9], a[8], a[11], a[10], a[13], a[12], a[15], a[14]]) }
}

#[inline]
fn mask_to_bits(mask: u128) -> Option<u8> {
    let mut digit = 0;
    let mut b = 128u8;

    for _ in 0..128 {
        let base = (15 - digit / 8) * 8;
        let offset = digit % 8;
        let index = base + offset;

        let n = (mask << index) >> 127;

        digit += 1;

        if n == 0 {
            b = (digit - 1) as u8;
            break;
        }
    }

    for digit in digit..128 {
        let base = (15 - digit / 8) * 8;
        let offset = digit % 8;
        let index = base + offset;

        if mask << index >> 127 == 1 {
            return None;
        }
    }

    Some(b)
}

// TODO: Ipv6Able
/// The type which can be token as an IPv6 address.
pub trait Ipv6Able {
    #[inline]
    fn get_u128(&self) -> u128;
}

impl Ipv6Able for u128 {
    #[inline]
    fn get_u128(&self) -> u128 {
        *self
    }
}

impl Ipv6Able for [u8; 16] {
    #[inline]
    fn get_u128(&self) -> u128 {
        u8_array_to_u128(*self)
    }
}

impl Ipv6Able for [u16; 8] {
    #[inline]
    fn get_u128(&self) -> u128 {
        u16_array_to_u128(*self)
    }
}

impl Ipv6Able for Ipv6Addr {
    #[inline]
    fn get_u128(&self) -> u128 {
        self.segments().get_u128()
    }
}

impl<T: Ipv6Able> Ipv6Able for &T {
    #[inline]
    fn get_u128(&self) -> u128 {
        Ipv6Able::get_u128(*self)
    }
}

// TODO: Ipv6Cidr

/// To represent IPv6 CIDR.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Ipv6Cidr {
    prefix: u128,
    mask: u128,
}

impl Debug for Ipv6Cidr {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let prefix = self.get_prefix_as_u16_array();
        let mask = self.get_mask_as_u16_array();
        let bits = self.get_bits();

        if f.alternate() {
            f.write_fmt(format_args!("Ipv6Cidr {{\n    prefix: {:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X},\n    mask: {:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X},\n    bits: {}\n}}", prefix[0], prefix[1], prefix[2], prefix[3], prefix[4], prefix[5], prefix[6], prefix[7], mask[0], mask[1], mask[2], mask[3], mask[4], mask[5], mask[6], mask[7], bits))
        } else {
            f.write_fmt(format_args!("{{ prefix: {:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}, mask: {:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}, bits: {} }}", prefix[0], prefix[1], prefix[2], prefix[3], prefix[4], prefix[5], prefix[6], prefix[7], mask[0], mask[1], mask[2], mask[3], mask[4], mask[5], mask[6], mask[7], bits))
        }
    }
}

impl Display for Ipv6Cidr {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let prefix = self.get_prefix_as_u16_array();
        let bits = self.get_bits();

        f.write_fmt(format_args!("{:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}:{:X}/{}", prefix[0], prefix[1], prefix[2], prefix[3], prefix[4], prefix[5], prefix[6], prefix[7], bits))
    }
}

impl PartialOrd for Ipv6Cidr {
    #[inline]
    fn partial_cmp(&self, other: &Ipv6Cidr) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Ipv6Cidr {
    #[inline]
    fn cmp(&self, other: &Ipv6Cidr) -> Ordering {
        let a = self.first_as_u16_array();
        let b = other.first_as_u16_array();

        for i in 0..16 {
            if a[i] > b[i] {
                return Ordering::Greater;
            } else if a[i] < b[i] {
                return Ordering::Less;
            }
        }

        self.get_bits().cmp(&other.get_bits())
    }
}

impl Ipv6Cidr {
    #[inline]
    pub fn get_prefix(&self) -> u128 {
        self.prefix
    }

    #[inline]
    pub fn get_prefix_as_u8_array(&self) -> [u8; 16] {
        u128_to_u8_array(self.get_prefix())
    }

    #[inline]
    pub fn get_prefix_as_u16_array(&self) -> [u16; 8] {
        u128_to_u16_array(self.get_prefix())
    }

    #[inline]
    pub fn get_prefix_as_ipv6_addr(&self) -> Ipv6Addr {
        let a = self.get_prefix_as_u16_array();

        Ipv6Addr::new(a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7])
    }

    #[inline]
    pub fn get_bits(&self) -> u8 {
        mask_to_bits(self.mask).unwrap()
    }

    #[inline]
    pub fn get_mask(&self) -> u128 {
        get_mask(self.get_bits())
    }

    #[inline]
    pub fn get_mask_as_u8_array(&self) -> [u8; 16] {
        u128_to_u8_array(self.get_mask())
    }

    #[inline]
    pub fn get_mask_as_u16_array(&self) -> [u16; 8] {
        u128_to_u16_array(self.get_mask())
    }

    #[inline]
    pub fn get_mask_as_ipv6_addr(&self) -> Ipv6Addr {
        let a = self.get_mask_as_u16_array();

        Ipv6Addr::new(a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7])
    }
}

#[derive(Debug, PartialEq)]
/// Possible errors of `Ipv6Cidr`.
pub enum Ipv6CidrError {
    IncorrectBitsRange,
    IncorrectMask,
    IncorrectIpv6CIDRString,
}

impl Ipv6Cidr {
    pub fn from_prefix_and_bits<P: Ipv6Able>(prefix: P, bits: u8) -> Result<Ipv6Cidr, Ipv6CidrError> {
        if bits > 128 {
            return Err(Ipv6CidrError::IncorrectBitsRange);
        }

        let mask = get_mask(bits);

        let prefix = prefix.get_u128() & mask;

        Ok(Ipv6Cidr {
            prefix,
            mask,
        })
    }

    pub fn from_prefix_and_mask<P: Ipv6Able, M: Ipv6Able>(prefix: P, mask: M) -> Result<Ipv6Cidr, Ipv6CidrError> {
        let mask = mask.get_u128();

        match mask_to_bits(mask) {
            Some(_) => {
                let prefix = prefix.get_u128() & mask;

                Ok(Ipv6Cidr {
                    prefix,
                    mask,
                })
            }
            None => {
                Err(Ipv6CidrError::IncorrectMask)
            }
        }
    }

//    pub fn from_str<S: AsRef<str>>(s: S) -> Result<Ipv6Cidr, Ipv6CidrError> {
//        let s = s.as_ref();
//
//        match RE_IPV4_CIDR.captures(s) {
//            Some(c) => {
//                let mut prefix = [0u8; 4];
//
//                prefix[0] = c.get(2).unwrap().as_str().parse().unwrap();
//                prefix[1] = c.get(5).map(|m| m.as_str().parse().unwrap()).unwrap_or(0);
//                prefix[2] = c.get(8).map(|m| m.as_str().parse().unwrap()).unwrap_or(0);
//                prefix[3] = c.get(11).map(|m| m.as_str().parse().unwrap()).unwrap_or(0);
//
//                if let Some(_) = c.get(13) {
//                    if let Some(m) = c.get(15) {
//                        Ok(Ipv6Cidr::from_prefix_and_bits(prefix, m.as_str().parse().unwrap())?)
//                    } else {
//                        let mut mask = [0u8; 4];
//
//                        mask[0] = c.get(20).unwrap().as_str().parse().unwrap();
//                        mask[1] = c.get(22).unwrap().as_str().parse().unwrap();
//                        mask[2] = c.get(24).unwrap().as_str().parse().unwrap();
//                        mask[3] = c.get(26).unwrap().as_str().parse().unwrap();
//
//                        Ok(Ipv6Cidr::from_prefix_and_mask(prefix, mask)?)
//                    }
//                } else {
//                    Ok(Ipv6Cidr::from_prefix_and_bits(prefix, 32)?)
//                }
//            }
//            None => {
//                Err(Ipv6CidrError::IncorrectIpv6CIDRString)
//            }
//        }
//    }
}

impl Ipv6Cidr {
    #[inline]
    pub fn first(&self) -> u128 {
        self.get_prefix()
    }

    #[inline]
    pub fn first_as_u8_array(&self) -> [u8; 16] {
        self.get_prefix_as_u8_array()
    }

    #[inline]
    pub fn first_as_u16_array(&self) -> [u16; 8] {
        self.get_prefix_as_u16_array()
    }

    #[inline]
    pub fn first_as_ipv6_addr(&self) -> Ipv6Addr {
        self.get_prefix_as_ipv6_addr()
    }

    #[inline]
    pub fn last(&self) -> u128 {
        !self.get_mask() | self.get_prefix()
    }

    #[inline]
    pub fn last_as_u8_array(&self) -> [u8; 16] {
        u128_to_u8_array(self.last())
    }

    #[inline]
    pub fn last_as_u16_array(&self) -> [u16; 8] {
        u128_to_u16_array(self.last())
    }

    #[inline]
    pub fn last_as_ipv6_addr(&self) -> Ipv6Addr {
        let a = self.last_as_u16_array();

        Ipv6Addr::new(a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7])
    }

    #[inline]
    pub fn size(&self) -> (u128, bool) {
        let bits = self.get_bits();

        if bits == 128 {
            (0, true)
        } else {
            (2u128.pow((128 - self.get_bits()) as u32), false)
        }
    }
}

impl Ipv6Cidr {
    #[inline]
    pub fn contains<IP: Ipv6Able>(&self, ipv6: IP) -> bool {
        let mask = self.get_mask();

        ipv6.get_u128() & mask == self.prefix
    }
}

// TODO: Ipv6CidrU8ArrayIterator

/// To iterate IPv6 CIDR.
#[derive(Debug)]
pub struct Ipv6CidrU8ArrayIterator {
    rev_from: u128,
    next: (u128, bool),
    size: (u128, bool),
}

impl Iterator for Ipv6CidrU8ArrayIterator {
    type Item = [u8; 16];

    #[inline]
    fn next(&mut self) -> Option<[u8; 16]> {
        if self.next == self.size {
            None
        } else {
            let p = self.rev_from + self.next.0;

            if self.next.0 == u128::max_value() {
                self.next = (0, true);
            } else {
                self.next.0 += 1;
            }

            let a = u128_to_u8_array(p);

            Some([a[15], a[14], a[13], a[12], a[11], a[10], a[9], a[8], a[7], a[6], a[5], a[4], a[3], a[2], a[1], a[0]])
        }
    }

    #[inline]
    fn last(mut self) -> Option<[u8; 16]> {
        self.next = (self.size.0 - 1, self.size.1);

        self.next()
    }
}

impl Ipv6Cidr {
    #[inline]
    pub fn iter_as_u8_array(&self) -> Ipv6CidrU8ArrayIterator {
        let a = self.get_prefix_as_u8_array();

        let rev_from = u8_array_to_u128([a[15], a[14], a[13], a[12], a[11], a[10], a[9], a[8], a[7], a[6], a[5], a[4], a[3], a[2], a[1], a[0]]);

        Ipv6CidrU8ArrayIterator {
            rev_from,
            next: (0, false),
            size: self.size(),
        }
    }
}