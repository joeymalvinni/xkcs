use crate::comic::{ComicIndex, ComicFrequency, Document};
use crate::utils::Field;

pub fn search(q: &str, doc: &mut Document) -> Vec<(f32, ComicIndex)> {
    // preprocessing on query
    let mut query = q.to_lowercase();
    query.retain(|c| !r#"(),".;:'"#.contains(c)); // strip punctuation

    let mut result = Vec::new();

    for comic in &doc.comics {
        let mut rank: f32 = 0.0;

        for word in query.split_whitespace() {
            // TODO: add transcript once transcript generated for all comics
            // rank += (tf-idf of alt * alt weight) + (tf-idf of title * title weight); where body weight + title weight = 1 <-------------------

            rank += calculate_tf(&word, &comic, &Field::Title) * calculate_idf(&word, &doc.frequency, &Field::Title, doc.comics.len()) * 0.6f32;
            rank +=  calculate_tf(&word, &comic, &Field::Alt) * calculate_idf(&word, &doc.frequency, &Field::Alt, doc.comics.len()) * 0.4f32; 
        }

        if rank < 0.0 {
            result.push((rank, comic.clone()));
        }
    }


    result.sort_by(|(a, _), (b, _)| a.partial_cmp(b).expect(&format!("{a} and {a} are not comparable")));

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
