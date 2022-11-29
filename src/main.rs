use std::{fs, path::PathBuf};

use anyhow::{Context, Error};
use clap::Parser;
use color::{Color, Style, COLORS};

mod color;

#[derive(clap::Parser)]
#[clap(author, version, about)]
struct Cli {
    file: PathBuf,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let input = fs::read_to_string(&cli.file).with_context(|| {
        format!(
            "Could not read input file at `{}`",
            cli.file.to_string_lossy()
        )
    })?;

    let output = parse(input);
    print!("{output}");
    Ok(())
}

pub fn parse(input: String) -> String {
    let mut out = String::new();
    let mut split = input.split('\x1b');
    out += split.next().expect("first part is always just text");

    let mut styles = vec![];

    for part in split {
        if !part.starts_with('[') {
            out += &("\x1b".to_owned() + part)
        }
        let Some((args, text)) = part.split_once('m') else {
            out += &("\x1b".to_owned() + part);
            continue;
        };

        let mut args = args[1..].split(';');
        while let Some(arg) = args.next() {
            if arg.is_empty() {
                styles.clear();
                continue;
            }
            match arg.parse::<u8>() {
                Ok(0) => styles.clear(),
                Ok(1) => styles.push(Style::Bold),
                Ok(3) => styles.push(Style::Italic),
                Ok(4) => styles.push(Style::Underline),
                Ok(22) => styles.retain(|s| !matches!(s, Style::Bold)),
                Ok(23) => styles.retain(|s| !matches!(s, Style::Italic)),
                Ok(24) => styles.retain(|s| !matches!(s, Style::Underline)),
                Ok(col @ 30..=37) => styles.push(Style::FgColor(Color::Simple(col - 30))),
                Ok(39) => styles.retain(|s| !matches!(s, Style::FgColor(_))),
                Ok(col @ 40..=47) => styles.push(Style::BgColor(Color::Simple(col - 40))),
                Ok(49) => styles.retain(|s| !matches!(s, Style::BgColor(_))),
                Ok(col @ 90..=97) => styles.push(Style::FgColor(Color::Simple(col - 82))),
                Ok(col @ 100..=107) => styles.push(Style::BgColor(Color::Simple(col - 92))),
                Ok(38) | Ok(48) => {
                    if let Some(arg2) = args.next() {
                        if let Ok(arg2) = arg2.parse::<u8>() {
                            if arg2 == 2 {
                                let r = if let Some(arg3) = args.next() {
                                    if let Ok(arg3) = arg3.parse::<u8>() {
                                        arg3
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                };
                                let g = if let Some(arg4) = args.next() {
                                    if let Ok(arg4) = arg4.parse::<u8>() {
                                        arg4
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                };
                                let b = if let Some(arg5) = args.next() {
                                    if let Ok(arg5) = arg5.parse::<u8>() {
                                        arg5
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                };
                                styles.push(if arg == "38" {
                                    Style::FgColor(Color::Rgb(r, g, b))
                                } else {
                                    Style::BgColor(Color::Rgb(r, g, b))
                                })
                            } else if arg2 == 5 {
                                if let Some(arg3) = args.next() {
                                    if let Ok(arg3) = arg3.parse::<u8>() {
                                        styles.push(if arg == "38" {
                                            Style::FgColor(Color::Simple(arg3))
                                        } else {
                                            Style::BgColor(Color::Simple(arg3))
                                        })
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            }
        }

        for line in text.split('\n') {
            let mut command = String::new();
            for style in &styles {
                match style {
                    Style::Bold => command += "×textbf{",
                    Style::Italic => command += "×textit{",
                    Style::Underline => command += "×uline{",
                    Style::FgColor(Color::Simple(code)) => {
                        command += &format!("×textcolor[HTML]{{{}}}{{", COLORS[*code as usize])
                    }
                    Style::FgColor(Color::Rgb(r, g, b)) => {
                        command += &format!("×textcolor[HTML]{{{r:02x}{g:02x}{b:02x}}}{{")
                    }
                    Style::BgColor(Color::Simple(code)) => {
                        command += &format!("×colorbox[HTML]{{{}}}{{", COLORS[*code as usize])
                    }
                    Style::BgColor(Color::Rgb(r, g, b)) => {
                        command += &format!("×colorbox[RGB]{{{r:02x},{g:02x},{b:02x}}}{{")
                    }
                }
            }
            out += &format!(
                "{command}{text}{braces}\n",
                text = line.replace('{', "×{").replace('}', "×}"),
                braces = "}".repeat(styles.len()),
            )
        }
        // remove last newline
        out.truncate(out.len() - 1);
    }

    out
}
