use std::{
    fmt::Debug,
    io::{self, BufRead},
};

enum CliError {
    IoError(io::Error),
    ParserError(podns::ParserError),
    Other(String),
}

impl Debug for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::IoError(e) => write!(f, "I/O Error - {}", e),
            CliError::ParserError(e) => write!(f, "Parser Error - {:?}", e),
            CliError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

fn main() -> Result<(), CliError> {
    // read from args, or fall back to stdin
    let sysargs = std::env::args().collect::<Vec<String>>();

    let domain = if sysargs.len() > 1 {
        sysargs[1].to_owned()
    } else {
        print!("Enter domain to resolve pronouns for (e.g. kinda.red): ");
        io::Write::flush(&mut io::stdout()).map_err(CliError::IoError)?;

        // read from stdin
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();
        handle.read_line(&mut line).map_err(CliError::IoError)?;
        line.trim().to_owned()
    };

    if domain.is_empty() {
        return Err(CliError::Other("No domain provided".to_string()));
    }

    match podns::resolve_pronouns(domain.as_str()) {
        Ok(mut records) => {
            if records.is_empty() {
                return Err(CliError::Other("No valid pronoun records found".to_string()));
            }

            // make sure records with the "preferred" tag are printed first
            records.sort_by(|a, b| a.set.cmp(&b.set));

            for record in records {
                println!("{}", record);
            }

            Ok(())
        }
        Err(e) => Err(CliError::IoError(e)),
    }
}
