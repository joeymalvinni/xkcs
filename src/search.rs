use std::io::{Write, stdout};

use crossterm::{
    execute,
    style::{self, Stylize}, cursor, terminal::{EnterAlternateScreen, LeaveAlternateScreen}
};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{event, terminal};
use std::time::Duration;

use crate::comic::{ComicIndex, ComicFrequency, Document};
use crate::utils::Field;
use crate::table;

pub fn search(q: &str, doc: &mut Document) -> Vec<(f32, ComicIndex)> {
    let mut query = q.to_lowercase();
    query.retain(|c| !r#"(),".;:'"#.contains(c)); // strip punctuation

    let mut result = Vec::new();

    for comic in &doc.comics {
        let mut rank: f32 = 0.0;

        for word in query.split_whitespace() {
            // TODO: add transcript once transcript generated for all comics
            // TODO: fix partial matches not working

            rank += calculate_tf(&word, &comic, &Field::Title) * calculate_idf(&word, &doc.frequency, &Field::Title, doc.comics.len()) * 0.6f32;
            rank +=  calculate_tf(&word, &comic, &Field::Alt) * calculate_idf(&word, &doc.frequency, &Field::Alt, doc.comics.len()) * 0.4f32; 
        }

        if rank < 0.0 {
            result.push((rank, comic.clone()));
        }
    }

    result.sort_by(|(a, c1), (b, c2)| if a == b {
        c1.comic.title.partial_cmp(&c2.comic.title).unwrap()
    } else {
        a.partial_cmp(b).expect(&format!("{a} and {a} are not comparable"))
    });


    result
}

fn calculate_tf(string: &str, c: &ComicIndex, field: &Field) -> f32 {
    match field {
        Field::Title => {
            let size = c.title_len as f32;
            let elements = c.title_freq.get(string).cloned().unwrap_or(0) as f32;

            elements / size
        },
        Field::Alt => {
            let size = c.alt_len as f32;
            let elements = c.alt_freq.get(string).cloned().unwrap_or(0) as f32;

            elements / size
        },
    }
}

fn calculate_idf(string: &str, df: &ComicFrequency, field: &Field, length: usize) -> f32 {
    match field {
        Field::Title => {
            let length = length as f32;
            let num = df.title_freq.get(string).cloned().unwrap_or(1) as f32;
            (num / length).log10()
        },
        Field::Alt => {
            let length = length as f32;
            let num = df.alt_freq.get(string).cloned().unwrap_or(1) as f32;
            (num / length).log10()
        },
    }
}

pub fn interactive_mode(doc: &mut Document) -> anyhow::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode().expect("Could not turn on Raw mode");
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, cursor::Hide)?;
    execute!(stdout, EnterAlternateScreen)?;

    let mut search_string = String::new();
    
    loop {
        if event::poll(Duration::from_millis(500)).expect("Error") {
            if let Event::Key(event) = event::read().expect("Failed to read line") { /* add this line */
                match event {
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: event::KeyModifiers::NONE,
                        kind: event::KeyEventKind::Press,
                        ..
                    } => {
                        match c {
                            _ => {
                                search_string.push(c);
                            },
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: event::KeyModifiers::CONTROL,
                        kind: event::KeyEventKind::Press,
                        ..
                    } => {
                        break;
                    }
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: event::KeyModifiers::SHIFT,
                        kind: event::KeyEventKind::Press,
                        ..
                    } => {
                        search_string.push(c);
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: event::KeyModifiers::NONE,
                        kind: event::KeyEventKind::Press,
                        ..
                    } => {
                        search_string.pop();
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: event::KeyModifiers::CONTROL,
                        kind: event::KeyEventKind::Press,
                        ..
                    } => {
                        search_string.clear();
                    }
                    _ => {}
                }

                execute!(stdout, cursor::MoveTo(0, 27))?;
                write!(stdout, "\x1b[KSearch: {}", search_string)?;

                let res = search(&search_string, doc);
                let top_20: Vec<(f32, ComicIndex)> = res.into_iter().take(20).collect();
                
                table::print_table(top_20);

                stdout.flush()?;
            };
        };
    }

    stdout.flush()?;
    execute!(stdout, LeaveAlternateScreen)?;

    terminal::disable_raw_mode().expect("Could not turn on Raw mode");
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, cursor::Show)?;
    Ok(())
}
