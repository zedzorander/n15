// Copyright © 2018 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

extern crate rand;
use rand::random;

use std::net::*;
use std::collections::HashSet;
use std::io::{BufRead, Write};
use std::fmt::{self, Display};

#[derive(Clone)]
struct Numbers(HashSet<u64>);

impl Display for Numbers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut elems: Vec<&u64> = self.0.iter().collect();
        elems.sort();
        let result: Vec<String> = elems
            .into_iter()
            .map(|i| i.to_string())
            .collect();
        let result = result.join(" ");
        write!(f, "{}", result)
    }
}

impl Numbers {

    fn new() -> Numbers {
        Numbers(HashSet::new())
    }

    fn insert(&mut self, e: u64) {
        assert!(self.0.insert(e));
    }

    fn remove(&mut self, e: u64) -> bool {
        self.0.remove(&e)
    }

    fn won(&self) -> Option<Numbers> {
        self.choose(3).into_iter().find(|Numbers(s)| {
            s.iter().sum::<u64>() == 15
        })
    }

    fn random_choice(&self) -> u64 {
        let choicevec: Vec<&u64> = self.0.iter().collect();
        let index = random::<usize>() % choicevec.len();
        *choicevec[index]
    }

    fn choose(&self, n: u64) -> Vec<Numbers> {
        let s = &self.0;
        if n == 0 || s.len() < n as usize {
            return Vec::new();
        }
        if s.len() == n as usize {
            return vec![Numbers(s.clone())];
        }
        let mut result: Vec<Numbers> = Vec::new();
        for e in s {
            let mut t = (*self).clone();
            t.remove(*e);
            result.extend(t.choose(n));
            let v: Vec<Numbers> = t.choose(n - 1)
                .into_iter()
                .map(|mut w| {
                    w.insert(*e);
                    w
                })
                .collect();
            result.extend(v);
        }
        result
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn game_loop<T: BufRead, U: Write>(mut reader: T, mut writer: U) -> Result<(), std::io::Error> {
    let mut unused = Numbers::new();
    for i in 1..=9 {
        unused.insert(i);
    }
    let mut you = Numbers::new();
    let mut me = Numbers::new();
    loop {
        writeln!(writer)?;
        writeln!(writer, "me: {}", me)?;
        writeln!(writer, "you: {}", you)?;
        writeln!(writer, "available: {}", unused)?;
        write!(writer, "move: ")?;
        writer.flush()?;
        let mut answer = String::new();
        reader.read_line(&mut answer)?;
        let n = answer.trim().parse::<u64>();
        let n = match n {
            Ok(n) => n,
            Err(_) => {
                writeln!(writer, "bad choice try again")?;
                continue;
            }
        };
        if !unused.remove(n) {
            writeln!(writer, "unavailable choice try again")?;
            continue;
        }
        you.insert(n);
        if let Some(win) = you.won() {
            writeln!(writer)?;
            writeln!(writer, "{}", win)?;
            writeln!(writer, "you win")?;
            return Ok(());
        }
        if unused.is_empty() {
            writeln!(writer, "draw")?;
            return Ok(());
        }
        let choice = unused.random_choice();
        writeln!(writer)?;
        writeln!(writer, "I choose {}", choice)?;
        unused.remove(choice);
        me.insert(choice);
        if let Some(win) = me.won() {
            writeln!(writer)?;
            writeln!(writer, "{}", win)?;
            writeln!(writer, "I win")?;
            return Ok(());
        }
        if unused.is_empty() {
            writeln!(writer, "draw")?;
            return Ok(());
        }
    }
}

fn main() {
    loop {
        let listener = TcpListener::bind("127.0.0.1:10015").unwrap();
        match listener.accept() {
            Ok((socket, addr)) => {
                println!("new client: {:?}", addr);
                let _ = std::thread::spawn(move || {
                    let reader = socket;
                    let mut writer = reader.try_clone().unwrap();
                    writeln!(writer, "n15 v0.0.0.1").unwrap();
                    let reader = std::io::BufReader::new(reader);
                    game_loop(reader, writer).unwrap();
                });
            },
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            },
        }
    }
}