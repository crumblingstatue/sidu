use std::io::{Read, Write};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let path = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("Missing path to file");
            print_help();
            return;
        }
    };
    let fmt_string = match args.next() {
        Some(arg) => match arg.to_str() {
            Some(str) => str.to_owned(),
            None => {
                eprintln!("Invalid (non-utf8) format string.");
                print_help();
                return;
            }
        },
        None => {
            eprintln!("Missing format string.");
            print_help();
            return;
        }
    };
    let format_cmds = parse_fmt(&fmt_string);
    let mut reader = std::io::BufReader::new(std::fs::File::open(path).unwrap());
    dump(&mut reader, &format_cmds);
}

fn dump(reader: &mut std::io::BufReader<std::fs::File>, format_cmds: &[FormatCmd]) {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let mut bytes = reader.bytes();
    for cmd in format_cmds.iter().cycle() {
        match cmd {
            FormatCmd::JustText(text) => {
                lock.write_all(text.as_bytes()).unwrap();
            }
            FormatCmd::NextByte(f) => {
                let b = match bytes.next() {
                    Some(byte) => byte.unwrap(),
                    None => {
                        return;
                    }
                };
                match f {
                    ByteFmt::Bin => write!(&mut lock, "{:b}", b).unwrap(),
                    ByteFmt::Dec => write!(&mut lock, "{}", b).unwrap(),
                    ByteFmt::Hex => write!(&mut lock, "{:x}", b).unwrap(),
                };
            }
        }
    }
}

fn parse_fmt(fmt_string: &str) -> Vec<FormatCmd> {
    let mut cmds = Vec::new();
    let mut text_start = 0;
    loop {
        let percent = match fmt_string[text_start..].find('%') {
            Some(pos) => pos,
            None => {
                // Nothing to do, except dump rest as text
                cmds.push(FormatCmd::JustText(&fmt_string[text_start..]));
                return cmds;
            }
        };
        // Dump text up to percent
        cmds.push(FormatCmd::JustText(&fmt_string[text_start..percent]));
        // Percent found, process it
        let spec = &fmt_string[percent + 1..percent + 2];
        let f = match spec {
            "b" => ByteFmt::Bin,
            "d" => ByteFmt::Dec,
            "h" => ByteFmt::Hex,
            _ => panic!("Invalid format specifier: {}", spec),
        };
        cmds.push(FormatCmd::NextByte(f));
        text_start += percent + 2;
    }
}

#[derive(Debug)]
enum FormatCmd<'s> {
    NextByte(ByteFmt),
    JustText(&'s str),
}

#[derive(Debug)]
enum ByteFmt {
    Bin,
    Dec,
    Hex,
}

fn print_help() {
    eprintln!(
        "sidu: simple dump tool
Usage: sidu <path> <fmt>

where:
path: Path to file to dump
fmt: Format string

Format string spec:
The format string can contain any character, plus special replacement characters
starting with %, which are as follows:
%d - Decimal byte
%h - Hex byte
%b - Binary byte
"
    );
}
