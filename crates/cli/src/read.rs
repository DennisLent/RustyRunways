use rustyline::{
    Context, Helper, Result as RustyResult,
    completion::{Completer, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

const KEYWORDS: &[&str] = &[
    "SHOW",
    "AIRPORTS",
    "WITH",
    "ORDERS",
    "PLANES",
    "PLANE",
    "BUY",
    "LOAD",
    "ORDER",
    "ORDERS",
    "UNLOAD",
    "ALL",
    "FROM",
    "ON",
    "DEPART",
    "HOLD",
    "ADVANCE",
    "CASH",
    "TIME",
    "STATS",
    "EXIT",
    "SparrowLight",
    "FalconJet",
    "CometRegional",
    "Atlas",
    "TitanHeavy",
    "Goliath",
    "Zephyr",
    "Lightning",
];

pub fn print_banner() {
    println!(
        r#"
     ███████████                       █████               ███████████                                                                     
    ░░███░░░░░███                     ░░███               ░░███░░░░░███                                                                    
    ░███    ░███  █████ ████  █████  ███████   █████ ████ ░███    ░███  █████ ████ ████████   █████ ███ █████  ██████   █████ ████  █████ 
    ░██████████  ░░███ ░███  ███░░  ░░░███░   ░░███ ░███  ░██████████  ░░███ ░███ ░░███░░███ ░░███ ░███░░███  ░░░░░███ ░░███ ░███  ███░░  
    ░███░░░░░███  ░███ ░███ ░░█████   ░███     ░███ ░███  ░███░░░░░███  ░███ ░███  ░███ ░███  ░███ ░███ ░███   ███████  ░███ ░███ ░░█████ 
    ░███    ░███  ░███ ░███  ░░░░███  ░███ ███ ░███ ░███  ░███    ░███  ░███ ░███  ░███ ░███  ░░███████████   ███░░███  ░███ ░███  ░░░░███
    █████   █████ ░░████████ ██████   ░░█████  ░░███████  █████   █████ ░░████████ ████ █████  ░░████░████   ░░████████ ░░███████  ██████ 
    ░░░░░   ░░░░░   ░░░░░░░░ ░░░░░░     ░░░░░    ░░░░░███ ░░░░░   ░░░░░   ░░░░░░░░ ░░░░ ░░░░░    ░░░░ ░░░░     ░░░░░░░░   ░░░░░███ ░░░░░░  
                                                ███ ░███                                                                 ███ ░███         
                                                ░░██████                                                                 ░░██████          
                                                ░░░░░░                                                                   ░░░░░░           
    "#
    );
}

pub struct LineReaderHelper {
    commands: Vec<String>,
}

impl LineReaderHelper {
    pub fn new() -> Self {
        let commands = KEYWORDS.iter().map(|&s| s.to_string()).collect();
        LineReaderHelper { commands }
    }
}

impl Helper for LineReaderHelper {}

impl Completer for LineReaderHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RustyResult<(usize, Vec<Pair>)> {
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |i| i + 1);

        let prefix = &line[start..pos].to_uppercase();

        let mut matches = Vec::new();
        for cmd in &self.commands {
            if cmd.to_uppercase().starts_with(prefix) {
                matches.push(Pair {
                    display: cmd.clone(),

                    replacement: cmd.clone(),
                });
            }
        }
        Ok((start, matches))
    }
}

impl Hinter for LineReaderHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Validator for LineReaderHelper {}

impl Highlighter for LineReaderHelper {}
