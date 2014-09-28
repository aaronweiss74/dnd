extern crate irc;
extern crate serialize;

use std::io::IoResult;
use data::{Player, World};

mod data;

fn str_to_u8(s: &str) -> u8 {
    match from_str(s) {
        Some(n) => n,
        None => 0,
    }
}

fn join_from(words: Vec<&str>, pos: uint) -> String {
    let mut res = String::new();
    for word in words.slice_from(pos).iter() {
        res.push_str(*word);
        res.push_char(' ');
    }
    let len = res.len() - 1;
    res.truncate(len);
    res
}

fn do_create(bot: &irc::Bot, resp: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() >= 3 {
        try!(bot.send_join(params[1]));
        let name = join_from(params.clone(), 2);
        try!(bot.send_topic(params[1], name.as_slice()));
        try!(bot.send_mode(params[1], "+i"));
        try!(world.add_game(name.as_slice(), resp, params[1]));
        try!(bot.send_privmsg(resp, format!("Campaign created named {}.", name).as_slice()));
        try!(bot.send_invite(resp, params[1]));
    } else {
        try!(bot.send_privmsg(resp, "Incorrect format for game creation. Format is:"));
        try!(bot.send_privmsg(resp, "create channel campaign name"));
    }
    Ok(())
}

fn do_register(bot: &irc::Bot, resp: &str, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 9 {
        let mut valid = true;
        for s in params.slice_from(3).iter() {
            if str_to_u8(*s) == 0 {
                valid = false;
            }
        }
        if valid {
            let p = try!(Player::create(params[1], params[2],
                str_to_u8(params[3]), str_to_u8(params[4]),
                str_to_u8(params[5]), str_to_u8(params[6]),
                str_to_u8(params[7]), str_to_u8(params[8])));
            try!(p.save());
            try!(bot.send_privmsg(resp, format!("Your account ({}) has been created.", params[1]).as_slice()));
        } else {
            try!(bot.send_privmsg(resp, "Stats must be non-zero positive integers. Format is: "))
            try!(bot.send_privmsg(resp, "register username password str dex con wis int cha"));
        }
    } else {
        try!(bot.send_privmsg(resp, "Incorrect format for registration. Format is:"));
        try!(bot.send_privmsg(resp, "register username password str dex con wis int cha"));
    }
    Ok(())
}

fn do_login(bot: &irc::Bot, resp: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 4 {
        let pr = Player::load(params[1]);
        if pr.is_ok() && !world.is_user_logged_in(resp) {
            let p = try!(pr);
            try!(world.add_user(resp, p.clone()));
            match world.games.find_mut(&String::from_str(params[3])) {
                Some(game) => {
                    let res = try!(game.login(p, resp, params[2]));
                    try!(bot.send_privmsg(resp, res));
                    if "Login successful.".eq(&res) {
                        try!(bot.send_invite(resp, params[3]));
                    };
                },
                None => try!(bot.send_privmsg(resp, format!("Game not found on channel {}.", params[2]).as_slice())),
            };
        } else if pr.is_err() {
            try!(bot.send_privmsg(resp, format!("Account {} does not exist, or could not be loaded.", params[1]).as_slice()));
        } else {
            try!(bot.send_privmsg(resp, "You can only be logged into one account at once."));
            try!(bot.send_privmsg(resp, "Use logout to log out."));
        }
    } else {
        try!(bot.send_privmsg(resp, "Incorrect format for login: Format is:"));
        try!(bot.send_privmsg(resp, "login username password channel"));
    }
    Ok(())
}

fn do_logout(bot: &irc::Bot, resp: &str, world: &mut World) -> IoResult<()> {
    if world.is_user_logged_in(resp) {
        try!(world.remove_user(resp));
        try!(bot.send_privmsg(resp, "You've been logged out."));
    } else {
        try!(bot.send_privmsg(resp, "You're not currently logged in."));
    }
    Ok(())
}

fn do_add_feat(bot: &irc::Bot, resp: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() > 1 {
        let res = world.get_user(resp);
        if res.is_ok() {
            let name = join_from(params.clone(), 1);
            let player = try!(res);
            player.add_feat(name.as_slice());
            try!(bot.send_privmsg(resp, format!("Added {} feat.", name).as_slice()));
        } else {
            try!(bot.send_privmsg(resp, "You must be logged in to add a feat."));
        }
    } else {
        try!(bot.send_privmsg(resp, "Can't add feat without a name. Format is:"));
        try!(bot.send_privmsg(resp, "addfeat name of feat"));
    }
    Ok(())
}

#[cfg(not(test))]
fn main() {
    let mut world = World::new().unwrap();
    let process = |bot: &irc::Bot, source: &str, command: &str, args: &[&str]| {
        match (command, args) {
            ("PRIVMSG", [chan, msg]) => {
                let resp = if chan.starts_with("#") {
                    chan
                } else {
                    match source.find('!') {
                        Some(i) => source.slice_to(i),
                        None => chan,
                    }
                };
                if !chan.starts_with("#") {
                    if msg.starts_with("register") {
                        try!(do_register(bot, resp.clone(), msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("login") {
                        try!(do_login(bot, resp.clone(), &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("create") {
                        try!(do_create(bot, resp.clone(), &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("logout") {
                        try!(do_logout(bot, resp.clone(), &mut world));
                    } else if msg.starts_with("addfeat") {
                        try!(do_add_feat(bot, resp.clone(), &mut world, msg.clone().split_str(" ").collect()));
                    }
                } else {
                    if msg.starts_with(".list") {
                        let mut s = String::new();
                        match bot.chanlists.find(&String::from_str(chan)) {
                            Some(vec) => {
                                for user in vec.iter() {
                                    s.push_str(user.as_slice());
                                    s.push_char(' ');
                                }
                                let len = s.len() - 1;
                                s.truncate(len);
                            },
                            None => {
                                s.push_str("None.");
                            }
                        }
                        try!(bot.send_privmsg(resp, s.as_slice()));
                    }
                }
            },
            _ => (),
        }
        Ok(())
    };
    let mut pickle = irc::Bot::new(process).unwrap();
    pickle.identify().unwrap();
    pickle.output();
}
