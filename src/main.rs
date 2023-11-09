#![warn(clippy::all, clippy::pedantic)]

use bracket_random::prelude::RandomNumberGenerator;
use colored::{Color, ColoredString, Colorize};
use std::collections::{HashMap, HashSet};

const ALL_WORDS: &str = include_str!("words.txt");

const WORD_LENGTH: usize = 5;
const MAX_TRIES: usize = 6;

const CORRECT_COLOR: Color = Color::BrightGreen;
const MISPLACED_COLOR: Color = Color::BrightYellow;
const INCORRECT_COLOR: Color = Color::BrightRed;

fn main() {
    let mut game = RustleGame::new();
    loop {
        game.display_guesses();
        let guess = game.ask_for_guess();
        if game.is_game_over(&guess) {
            break;
        }
    }
}

fn words_list() -> Vec<String> {
    ALL_WORDS
        .split('\n')
        .skip(2)
        .map(sanitize_word)
        .filter(|line| line.len() == WORD_LENGTH)
        .collect()
}

fn sanitize_word(word: &str) -> String {
    word.trim()
        .to_uppercase()
        .chars()
        .filter(char::is_ascii_alphabetic)
        .collect()
}

struct RustleGame {
    dictionary: Vec<String>,
    word: String,
    guessed_letters: HashSet<char>,
    guesses: Vec<Vec<ColoredString>>,
}

impl RustleGame {
    fn new() -> Self {
        let mut rng = RandomNumberGenerator::new();
        let dictionary = words_list();
        let word = rng.random_slice_entry(&dictionary).unwrap().clone();
        Self {
            dictionary,
            word,
            guessed_letters: HashSet::new(),
            guesses: Vec::new(),
        }
    }

    fn colorize_guess(&mut self, guess: &str) -> Vec<ColoredString> {
        let mut guess_array: Vec<ColoredString> = guess
            .chars()
            .map(|c| c.to_string().color(INCORRECT_COLOR))
            .collect();

        let mut char_count: HashMap<char, i32> = create_charmap(&self.word);

        // Set the correct letters to green
        guess.chars().enumerate().for_each(|(pos, c)| {
            if self.word.chars().nth(pos).unwrap() == c {
                guess_array[pos] = format!("{c}").color(CORRECT_COLOR);
                char_count.entry(c).and_modify(|e| *e -= 1);
            };
        });

        // Set the correct but misplaced letters to yellow
        guess.chars().enumerate().for_each(|(pos, c)| {
            if is_color(&guess_array[pos], CORRECT_COLOR) {
                return;
            };

            self.word.chars().enumerate().for_each(|(i, ch)| {
                if is_color(&guess_array[i], CORRECT_COLOR) {
                    return;
                };

                if ch == c && char_count[&c] > 0 {
                    guess_array[pos] = format!("{c}").color(MISPLACED_COLOR);
                    char_count.entry(c).and_modify(|e| *e -= 1);
                }
            });
        });

        guess.chars().enumerate().for_each(|(pos, c)| {
            if !is_color(&guess_array[pos], CORRECT_COLOR)
                && !is_color(&guess_array[pos], MISPLACED_COLOR)
            {
                self.guessed_letters.insert(c);
            };
        });
        guess_array
    }

    fn display_guesses(&mut self) {
        self.guesses
            .iter()
            .enumerate()
            .for_each(|(guess_number, guess)| {
                print!("{}: ", guess_number + 1);
                for elem in guess {
                    print!("{elem} ");
                }
                println!();
            });
    }

    fn display_invalid_letters(&self) {
        if !self.guessed_letters.is_empty() {
            print!("Letters not in the word: ");
            self.guessed_letters
                .iter()
                .for_each(|letter| print!("{letter} "));
            println!();
        }
    }

    fn ask_for_guess(&mut self) -> String {
        println!(
            "{}",
            format!("Enter your word guess ({WORD_LENGTH} letters) and press ENTER").cyan()
        );
        self.display_invalid_letters();
        let mut guess = String::new();
        let mut valid_guess = false;
        while !valid_guess {
            guess = String::new();
            std::io::stdin().read_line(&mut guess).unwrap();
            guess = sanitize_word(&guess);
            if guess.len() != WORD_LENGTH {
                println!(
                    "{}",
                    format!("Your guess must be {WORD_LENGTH} letters.").red()
                );
            } else if !self.dictionary.iter().any(|word| word == &guess) {
                println!(
                    "{}",
                    format!("{guess} isn't in the Rustle dictionary.").red()
                );
            } else {
                let c_string = self.colorize_guess(&guess);
                self.guesses.push(c_string);
                valid_guess = true;
            }
        }
        guess
    }

    fn is_game_over(&self, guess: &str) -> bool {
        let n_tries = self.guesses.len();
        if guess == self.word {
            println!("Correct! You guessed the word in {n_tries} tries.");
            true
        } else if n_tries >= MAX_TRIES {
            println!(
                "{}",
                format!("You ran out of tries! The word was {}", self.word).bright_red()
            );
            true
        } else {
            false
        }
    }
}

fn create_charmap(word: &str) -> HashMap<char, i32> {
    let mut charmap: HashMap<char, i32> = HashMap::new();
    word.chars().for_each(|c| {
        let count = charmap.entry(c).or_insert(0);
        *count += 1;
    });
    charmap
}

fn is_color(c: &ColoredString, col: Color) -> bool {
    if *c == c.clone().color(col) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_word() {
        assert_eq!(sanitize_word("hello"), "HELLO");
        assert_eq!(sanitize_word("hello world"), "HELLOWORLD");
    }

    #[test]
    fn test_is_color() {
        let c = "a".bright_green();
        assert!(is_color(&c, Color::BrightGreen));
        assert!(!is_color(&c, Color::Blue));
    }

    #[test]
    fn test_create_charmap() {
        let word = "hello";
        let charmap = create_charmap(word);
        assert_eq!(charmap[&'h'], 1);
        assert_eq!(charmap[&'e'], 1);
        assert_eq!(charmap[&'l'], 2);
        assert_eq!(charmap[&'o'], 1);
    }

    #[test]
    fn test_colorize_guess_all_correct() {
        let mut game = RustleGame::new();
        game.word = "ABC".to_string();
        let colored_guess = game.colorize_guess(&("ABC".to_string()));
        assert_eq!(
            colored_guess,
            (vec![
                "A".color(CORRECT_COLOR),
                "B".color(CORRECT_COLOR),
                "C".color(CORRECT_COLOR)
            ])
        );
        assert_eq!(game.guessed_letters, (vec![]).into_iter().collect());
    }

    #[test]
    fn test_colorize_guess_all_incorrect() {
        let mut game = RustleGame::new();
        game.word = "ABC".to_string();
        let colored_guess = game.colorize_guess(&("DEF".to_string()));
        assert_eq!(
            colored_guess,
            (vec![
                "D".color(INCORRECT_COLOR),
                "E".color(INCORRECT_COLOR),
                "F".color(INCORRECT_COLOR)
            ])
        );
        assert_eq!(
            game.guessed_letters,
            (vec!['D', 'E', 'F'].into_iter().collect())
        );
    }

    #[test]
    fn test_colorize_guess_two_misplaced() {
        let mut game = RustleGame::new();
        game.word = "ABC".to_string();
        let colored_guess = game.colorize_guess(&("ACB".to_string()));
        assert_eq!(
            colored_guess,
            (vec![
                "C".color(CORRECT_COLOR),
                "B".color(MISPLACED_COLOR),
                "A".color(MISPLACED_COLOR)
            ])
        );
        assert_eq!(game.guessed_letters, (vec!['B', 'A'].into_iter().collect()));
    }

    #[test]
    fn test_colorize_guess_one_correct_one_misplaced() {
        let mut game = RustleGame::new();
        game.word = "ABC".to_string();
        let colored_guess = game.colorize_guess(&("ACD".to_string()));
        assert_eq!(
            colored_guess,
            (vec![
                "A".color(CORRECT_COLOR),
                "C".color(MISPLACED_COLOR),
                "D".color(INCORRECT_COLOR)
            ])
        );
        assert_eq!(game.guessed_letters, (vec!['D'].into_iter().collect()));
    }
}
