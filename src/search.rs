use std::io::{Write, stdout}; use std::time::Duration;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    cursor
};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{event, terminal};
use ndarray::{Array1, Zip};

use crate::comic::{Comic, ComicIndex, ComicFrequency, Document};
use crate::utils::Field;
use crate::table;

pub fn search(q: &str, doc: &mut Document) -> Vec<(f32, Comic)> {
    let mut query = q.to_lowercase();
    query.retain(|c| !r#"(),".;:'"#.contains(c)); // strip punctuation

    let mut result = Vec::new();

    for comic in &doc.comics {
        let mut rank: f32 = 0.0;
        for word in query.split_whitespace() {
            // TODO: add transcript once transcript generated for all comics

            rank += calculate_tf(&word, &comic, &Field::Title) * calculate_idf(&word, &doc.frequency, &Field::Title, doc.comics.len()) * 0.6f32;
            rank += calculate_tf(&word, &comic, &Field::Alt) * calculate_idf(&word, &doc.frequency, &Field::Alt, doc.comics.len()) * 0.4f32;
        }

        if rank < 0.0 {
            result.push((rank, comic.comic.clone()));
        } else {
            let text = vec![comic.comic.title.clone(), comic.comic.alt.clone()];

            let fuzzy_scores = fuzzy_find(query.clone(), text);
            for (_, similarity) in fuzzy_scores {
                rank += similarity as f32;
            }

            if rank > 0.0 {
                result.push(((-rank)/8.0, comic.comic.clone()));
            }
        }
    }

    result.sort_by(|(a, c1), (b, c2)| if a == b {
        c1.title.partial_cmp(&c2.title).unwrap()
    } else {
        a.partial_cmp(b).expect(&format!("{a} and {a} are not comparable"))
    });

    result
}

fn cosine_similarity(vec1: &Array1<f64>, vec2: &Array1<f64>) -> f64 {
    let dot_product = Zip::from(vec1).and(vec2).fold(0.0, |acc, &a, &b| acc + a * b);
    let norm1 = vec1.dot(vec1).sqrt();
    let norm2 = vec2.dot(vec2).sqrt();

    if norm1 * norm2 == 0.0 {
        0.0 // division by zero
    } else {
        dot_product / (norm1 * norm2)
    }
}

fn vectorize_string(s: &str) -> Array1<f64> {
    let mut vector = Array1::zeros(96); 

    for c in s.chars() {
        if let Some(index) = c.to_digit(36) {
            let idx = (index % 96) as usize;
            vector[idx] += 1.0;
        }
    }

    vector
}

fn fuzzy_find(target: String, candidates: Vec<String>) -> Vec<(String, f64)> {
    let target_vector = vectorize_string(&target);

    let mut results: Vec<(String, f64)> = Vec::new();

    for candidate in candidates {
        let candidate_vector = vectorize_string(&candidate);
        let similarity = cosine_similarity(&target_vector, &candidate_vector);
        results.push((candidate, similarity));
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results
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

pub fn interactive_mode(doc: &mut Document, lim: usize) -> anyhow::Result<()> {
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
                        code: KeyCode::Esc,
                        modifiers: event::KeyModifiers::NONE,
                        kind: event::KeyEventKind::Press,
                        ..
                    } => {
                        search_string.clear();
                    }
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
                    _ => {}
                }

                execute!(stdout, cursor::MoveTo(0, lim as u16 + 5))?;
                write!(stdout, "\x1b[KSearch: {}", search_string)?;

                let res = search(&search_string, doc);
                let top_20: Vec<(f32, Comic)> = res.into_iter().take(lim).collect();
                
                table::print_table(top_20, lim);

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
