use crate::comic;
use crate::utils::{RED, GREEN, MAGENTA, CYAN, RESET};
use std::io::{stdout};
use crossterm::{execute, cursor, terminal};
use tabled::{
    settings::{
        style::{HorizontalLine, Style, VerticalLine},
        Alignment,
        Modify,
        Color
    },
    Table, Tabled,
};

#[derive(Tabled)]
pub struct ComicTable {
    rank: String, 
    title: String,
    alternate: String,
    link: String
}

pub fn print_table(results: Vec<(f32, comic::Comic)>) {
    let mut stdout = stdout();
    let mut t: Vec<ComicTable> = Vec::new();

    for (rank, com) in results {
        let mut a: String = com.alt.chars().take(20).collect::<String>();
        a.push_str("...");
        t.push(ComicTable {
            // truncate decimal hack
            rank: String::from(format!("{:.4}", -rank)),
            title: com.title,
            alternate: a,
            // this implementation might possibly be the worst way to do this
            link: String::from(format!("\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\", format!("https://xkcd.com/{}", com.num), format!("xkcd/{}", com.num)))
        })
    }

    let theme = Style::modern()
        .horizontals([(1, HorizontalLine::inherit(Style::modern()))])
        .verticals([(1, VerticalLine::inherit(Style::modern()))])
        .remove_horizontal()
        .remove_vertical();

    let mut table = Table::new(t);
    table
        .with(theme)
        .with(Modify::new((0, 0)).with(Color::FG_MAGENTA))
        .with(Modify::new((0, 1)).with(Color::FG_CYAN))
        .with(Modify::new((0, 2)).with(Color::FG_GREEN))
        .with(Modify::new((0, 3)).with(Color::FG_RED));

    let output = table.to_string().replace("\n", "\n\r");
    execute!(stdout, cursor::MoveTo(0, 24)).expect("Failed to execute command");
    execute!(stdout, terminal::Clear(terminal::ClearType::FromCursorUp)).expect("Failed to execute command");
    execute!(stdout, cursor::MoveTo(0, 0)).expect("Failed to execute command");
    print!("{output}");
}
