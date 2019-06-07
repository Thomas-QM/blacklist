extern crate blacklist_lib;
use blacklist_lib::*;

extern crate serenity;
extern crate onig;
extern crate dotenv;

use std::env;
use onig::{Regex};
use serenity::prelude::*;
use serenity::model::channel::Message;

//stolen from https://stackoverflow.com/questions/3809401/what-is-a-good-regular-expression-to-match-a-url (non-restrictive version)
const LINK_REGEX: &str = r"(http(s)?:\/\/.)?(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)";

struct Handler {
    blacklist: Blacklist,
    link_regex: Regex
}

impl EventHandler for Handler {
    fn message(&self, _ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }

        let guild = match message.guild_id {
            Some(x) => x, _ => return
        };

        for (from, to) in self.link_regex.find_iter(&message.content) {            
            if check(&message.content[from..to], &self.blacklist) {
                //if the bot doesnt have perms, it wont do it... easy config :)
                let _ = message.delete();
                let _ = guild.kick(message.author.id);
            }
        }
    }
}

fn main() {
    dotenv::dotenv().unwrap();

    let handler = Handler {
        blacklist: load(),
        link_regex: Regex::new(LINK_REGEX).unwrap()
    };

    println!("Loaded.");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").unwrap(), handler)
        .expect("Error creating client");

    client.start_autosharded().unwrap();
}