use crate::libfukushuu::shitsumon::{OptionPair, Question};
use crate::{Choice, Error};
use colored::Colorize;
#[cfg(feature = "kittygfx")]
use kitty_image::{Action, Command, WrappedCommand};
use log::debug;
use rusqlite::Connection;
use rusqlite::Result;
use text_io::read;

pub fn cli_loop(
    conn: &Connection,
    questions: Vec<Question>,
    question_count: u32,
    choices_count: u32,
) -> Result<(), Error> {
    for idx in 1..questions.len() + 1 {
        macro_rules! incr_and_print {
            ($to_incr: expr) => {
                let score = $to_incr.increment_score(&conn).unwrap();
                println!(
                    "{}",
                    format!("Correct!: {} -> {}", score - 1, score).bright_green()
                )
            };
        }
        macro_rules! decr_and_print {
            ($to_incr: expr) => {
                let score = $to_incr.decrement_score(&conn).unwrap();
                println!(
                    "{}",
                    format!("Incorrect!: {} -> {}", score + 1, score).bright_red()
                )
            };
        }

        let leading = format!("{}/{}. ", idx, question_count);
        println!(
            "{}{}",
            leading.cyan(),
            format!(
                "{:?} ({})",
                questions[idx - 1].front,
                questions[idx - 1].score
            )
            .black()
            .bold()
            .on_white()
        );
        let (options, correct) = questions[idx - 1].get_options_randomize();

        let indent = " ".repeat(leading.len());
        for (i, OptionPair(str, img)) in options.iter().enumerate() {
            print!("{}{}. ", indent, format!("{}", i + 1).bold());
            if let Some(string) = str {
                println!("{}", string);
            }

            #[cfg(feature = "kittygfx")]
            if let Some(image_path) = img {
                debug!("path at {image_path:?}");
                let (width, height) = image::image_dimensions(image_path)?;
                let x_offset = if str.is_some() {
                    10 * leading.len() as u32
                } else {
                    0
                };
                let action = Action::TransmitAndDisplay(
                    kitty_image::ActionTransmission {
                        format: kitty_image::Format::Png,
                        medium: kitty_image::Medium::File,
                        width,
                        height,
                        ..Default::default()
                    },
                    kitty_image::ActionPut {
                        x_offset,
                        ..Default::default()
                    },
                );
                let command =
                    WrappedCommand::new(Command::with_payload_from_path(action, image_path));
                println!("{command}");
                print!("{}", "\n".repeat(height as usize / 20));
            }
        }

        print!(
            "{} ",
            "Answer (1-4, q to quit prematurely and anything else if you don't know):".cyan()
        );
        let choice_string: String = read!("{}\n");
        let choice = Choice::from_str(choices_count, choice_string.as_str());
        debug!("choice: {:?}", choice);

        match choice {
            Choice::Option(num) => {
                if num == correct {
                    incr_and_print!(questions[idx - 1]);
                } else {
                    decr_and_print!(questions[idx - 1]);
                    println!(
                        "{}",
                        format!("The correct choice was {:?}.", correct).green()
                    )
                }
            }
            Choice::DontKnow => {
                decr_and_print!(questions[idx - 1]);
                println!(
                    "{}",
                    format!("The correct choice was {:?}.", correct).green()
                )
            }
            Choice::Quit => {
                println!("{}", "Quitting Early!".cyan());
                return Ok(());
            }
        }
    }
    Ok(())
}
