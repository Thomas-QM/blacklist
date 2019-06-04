extern crate serde;
extern crate metrohash;

use serde::*;
use std::fs;
use std::cmp::Ordering;

#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
//whitelistdomain > domain > startswith > endswith > urlstartswith
pub enum BlacklistMode {
     WhiteListDomain, Domain, StartsWith, UrlStartsWith, EndsWith
}


#[derive(Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum BlacklistExtItem {
    WhiteListDomain(String), Domain(String),
    StartsWith(String), UrlStartsWith(String), EndsWith(String)
}

impl BlacklistExtItem {
    pub fn to_bi(&self) -> BlacklistItem {
        match self {
            BlacklistExtItem::WhiteListDomain(domain) => BlacklistItem(BlacklistMode::WhiteListDomain, hash(domain)),
            BlacklistExtItem::Domain(domain) => BlacklistItem(BlacklistMode::Domain, hash(domain)),
            BlacklistExtItem::StartsWith(sw) => BlacklistItem(BlacklistMode::StartsWith, hash(sw)),
            BlacklistExtItem::UrlStartsWith(url) => BlacklistItem(BlacklistMode::UrlStartsWith, hash(url)),
            BlacklistExtItem::EndsWith(ew) => BlacklistItem(BlacklistMode::EndsWith, hash(ew))
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct BlacklistItem(pub BlacklistMode, pub u64);

impl PartialOrd for BlacklistItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlacklistItem {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0.clone() as u8).cmp(&(other.0.clone() as u8))
    }
}

pub type Blacklist = Vec<BlacklistItem>;

pub fn hash(s: &str) -> u64 {
    use metrohash::MetroHash;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = MetroHash::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub fn write(bl: &Blacklist) {
    let mut buf = Vec::new();
    
    for BlacklistItem(bm, bi) in bl {
        buf.push(bm.clone() as u8);
        buf.extend_from_slice(&bi.to_ne_bytes());
    }
    
    fs::write("./blacklist", buf).unwrap();
}

pub fn load() -> Blacklist {
    let mut bl = Vec::new();
    let mut buf =  fs::read("./blacklist").unwrap().into_iter();

    loop {
        if let Some(n) = buf.next() {
            let bm: BlacklistMode = unsafe { std::mem::transmute(n) };
            
            let mut bh = [0; 8];
            for i in 0..8 {
                bh[i] = buf.next().unwrap();
            }

            bl.push(BlacklistItem(bm, u64::from_ne_bytes(bh)));
        } else {
            break;
        }
    }

    bl
}

pub fn check(url: &str, bl: &Blacklist) -> bool {
    let len = url.len(); //optimize static call not really but also abbreviation

    let domain_start = url.find("://").map(|x| x+3).unwrap_or(0);
    
    let mut separators: Vec<usize> =
        url[domain_start..].char_indices().filter_map(|(i, c)| {
            if c == '/' || c == '.' {
                Some(domain_start+i)
            } else {
                None
            }
        }).collect();

    separators.push(domain_start);
    separators.push(url.len());

    let domain_end = url[domain_start..].find('/').map(|x| x+domain_start).unwrap_or(len);

    let domain = hash(&url[domain_start..domain_end]);

    let mut start_ranges = Vec::new();
    let mut end_ranges = Vec::new();
    
    if len-domain_start > 0 {
        for sep in &separators {
            start_ranges.push(hash(&url[domain_start..*sep]));
        }

        for sep in &separators {
            end_ranges.push(hash(&url[*sep..]));
        }
    }

    let mut url_ranges = Vec::new();
    if len > 0 {
        for x in 1..len {
            url_ranges.push(hash(&url[0..x]));
        }
    }

    for bi in bl {
        let x = bi.1;
        let res = match bi.0 {
            BlacklistMode::WhiteListDomain =>
                if x == domain { return false } else { false },

            BlacklistMode::Domain => domain == x,
            BlacklistMode::StartsWith => start_ranges.contains(&x),
            BlacklistMode::UrlStartsWith => url_ranges.contains(&x),
            BlacklistMode::EndsWith => end_ranges.contains(&x)
        };

        if res {
            return true;
        }
    }

    false
}