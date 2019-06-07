use blacklist_lib::*;
//globglobgalab
use serde::*;
use reqwest::*;
use rayon::prelude::*;

const SUPPORTED_SYNTAXES: [u8; 5] = [1, 2, 8, 16, 20];

fn interpret(bl: &mut Blacklist, i: u8, whitelist: bool, s: &str) {
	fn hostlike_syntax<F: FnMut(&str)>(s: &str, mut f: F) {
		for line in s.split('\n') {
			//skip if comment or completely spaces
			if line.starts_with('#') || line.trim_start_matches(' ').len() == 0 { continue; }
			if line == "404: Not Found" { //github raw content not found
				break;
			}

			f(line.trim_end_matches('\r'));
		}
	}

	match i {
		1 => { //hosts file
			hostlike_syntax(s, |mut line| {
				const HOST_PREFIX_1: &str = "127.0.0.1";
				const HOST_PREFIX_2: &str = "0.0.0.0";

				if line.starts_with(HOST_PREFIX_1) {
					line = &line[HOST_PREFIX_1.len()+1..];
				} else if line.starts_with(HOST_PREFIX_2) {
					line = &line[HOST_PREFIX_2.len()+1..];
				}

				if whitelist {
					bl.push(BlacklistItem(BlacklistMode::WhiteListDomain, hash(line)));
				} else {
					bl.push(BlacklistItem(BlacklistMode::Domain, hash(line)));
				}
			});
		}, 2 => {
			hostlike_syntax(s, |line| {
				if whitelist {
					bl.push(BlacklistItem(BlacklistMode::WhiteListDomain, hash(line)));
				} else {
					bl.push(BlacklistItem(BlacklistMode::Domain, hash(line)));
				}
			});
		}, 8 => {
			hostlike_syntax(s, |line| {
				bl.push(BlacklistItem(BlacklistMode::UrlStartsWith, hash(line)));
			});
		}, 16 => {
			const WILDCARD: char = '*';

			hostlike_syntax(s, |line| {
				if !line.contains(WILDCARD) {
					bl.push(BlacklistItem(BlacklistMode::Domain, hash(line)));
				} else if line.starts_with(WILDCARD) {
					bl.push(BlacklistItem(BlacklistMode::EndsWith, hash(&line[1..])));
				} else if line.ends_with(WILDCARD) {
					bl.push(BlacklistItem(BlacklistMode::StartsWith, hash(&line[0..line.len()-1])));
				}
			});
		}, 20 => {
			hostlike_syntax(s, |mut line| {
				line = line.trim_start_matches("address=/");
				
				line = line.trim_end_matches("/::");
				line = line.trim_end_matches("/0.0.0.0");

				bl.push(BlacklistItem(BlacklistMode::Domain, hash(line)));
			});
		}, 
		
		_ => ()
	}
}

const FILTERLISTS: &str = "https://filterlists.com/api/v1/lists";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilterListsItem {
	syntax_id: Option<u8>,
    view_url: String,
	name: String
}

pub fn scrape(bl: &mut Blacklist) {
	{
		let items = get(FILTERLISTS).unwrap()
			.json::<Vec<FilterListsItem>>().unwrap();

		let items: Vec<(u8, bool, String)> =
			items.into_par_iter().filter_map(|x| {
				if let Some(syntax) = x.syntax_id {
					if SUPPORTED_SYNTAXES.contains(&syntax) {
						println!("Fetching {}", x.name);

						match get(&x.view_url).and_then(|mut x| x.text()) {
							Ok(resp) => return Some((syntax, x.view_url.contains("whitelist"), resp)),
							Err(err) => println!("Error fetching from {}: {}", &x.view_url, err)
						}
					}
				}

				None
			})
			.collect();

		println!("Compiling...");
		for (syntax, whitelist, resp) in items {
			interpret(bl, syntax, whitelist, &resp);
		}
	}
}