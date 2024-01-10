use crate::comic;
use crate::utils::{RED, GREEN, MAGENTA, CYAN, RESET};
use std::io::{stdout};
use crossterm::{execute, cursor, terminal};

pub fn print_table(results: Vec<(f32, comic::ComicIndex)>) {
    let mut stdout = stdout();
    let padding = 4;
    let max_rank_len = 6 + padding;
    let max_name_len = results.iter().map(|(_, c)| c.comic.title.len()).max().unwrap_or(0).max(6) + padding;
    let max_num_len = results.iter().map(|(_, c)| c.comic.num.to_string().len()).max().unwrap_or(0).max(13) + padding;

    let mut output = String::new();

    // Append to the output string
    output += &format!("┌{0:─<width_rank$}┬{0:─<width_rank$}┬{0:─<width_name$}┬{0:─<width_num$}┐\n\r", "", width_rank = max_rank_len, width_name = max_name_len, width_num = max_num_len);
    output += &format!("│{MAGENTA} {:<width_rank$}{RESET}│{CYAN} {:<width_rank$}{RESET}│{GREEN} {:<width_name$}{RESET}│{RED} {:<width_num$}{RESET}│\n\r", "Index", "Rank", "Title", "Comic Number", width_rank = max_rank_len-1, width_name = max_name_len-1, width_num = max_num_len-1);
    output += &format!("├{0:─<width_rank$}┼{0:─<width_rank$}┼{0:─<width_name$}┼{0:─<width_num$}┤\n\r", "", width_rank = max_rank_len, width_name = max_name_len, width_num = max_num_len);

    for (index, (rank, c)) in results.iter().enumerate() {
        output += &format!(
            "│ {:<width_rank$}│ {:<width_rank$.4}│ {:<width_name$}│ {:<width_num$}│\n\r",
            index + 1,
            -rank,
            c.comic.title,
            c.comic.num,
            width_rank = max_rank_len-1,
            width_name = max_name_len-1,
            width_num = max_num_len-1,
        );
    }

    output += &format!("└{0:─<width_rank$}┴{0:─<width_rank$}┴{0:─<width_name$}┴{0:─<width_num$}┘\n\r", "", width_rank = max_rank_len, width_name = max_name_len, width_num = max_num_len);

    execute!(stdout, cursor::MoveTo(0, 25)).expect("Failed to execute command");
    execute!(stdout, terminal::Clear(terminal::ClearType::FromCursorUp)).expect("Failed to execute command");
    execute!(stdout, cursor::MoveTo(0, 0)).expect("Failed to execute command");
    print!("{}", output);
}
