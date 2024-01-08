use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use futures::{stream, StreamExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use anyhow;
use reqwest;

const PARALLEL_REQUESTS: usize = 8;
const URL: &str = "https://xkcd.com";
const INFO: &str = "info.0.json";

const DATA_PATH: &str = "data/document.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Comic {
    month: String,
    num: u16,
    year: String,
    day: String,
    title: String,
    alt: String,
    img: String,
    transcript: String,
}

// TODO: also use transcripts for searching
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ComicIndex {
    title_freq: HashMap<String, usize>,
    title_len: usize,
    alt_freq: HashMap<String, usize>,
    alt_len: usize,
    comic: Comic,
}

enum Field {
    Title,
    Alt,
    // Transcript
}

// all frequencies for document frequencies
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ComicFrequency {
    alt_freq: HashMap<String, usize>,
    title_freq: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Document {
    comics: Vec<ComicIndex>,
    frequency: ComicFrequency
}


async fn download_all(latest: Comic, client: &reqwest::Client) -> anyhow::Result<()> {
    let c = Mutex::new(Vec::new());
    
    let frequencies = Arc::new(Mutex::new(ComicFrequency {
        title_freq: HashMap::new(),
        alt_freq: HashMap::new(),
    }));

    let indices = (1..=latest.num).collect::<Vec<_>>();

    let bodies = stream::iter(indices)
        .map(|index| {
            let url = format!("{URL}/{index}/{INFO}");
            let client = client.clone();
            let frequencies = Arc::clone(&frequencies);

            tokio::spawn(async move {
                let resp = client.get(&url).send().await?;
                let mut parsed = resp.json::<Comic>().await?;

                index_comic(&mut parsed, &mut *frequencies.lock().await).await
            })
        })
        .buffer_unordered(PARALLEL_REQUESTS);
    bodies
        .for_each(|x| async {
            match x {
                Ok(Ok(x)) => {
                    c.lock().await.push(x);
                },
                Ok(Err(e)) => eprintln!("Got a reqwest::Error: {}", e),
                Err(e) => eprintln!("Got a tokio::JoinError: {}", e),
            }
        })
        .await;

    let c = c.lock().await;
    let frequencies = frequencies.lock().await.clone();

    let doc = Document {
        comics: (*c).to_vec(),
        frequency: frequencies,
    };

    let file = File::create(DATA_PATH)?;
    serde_json::to_writer(file, &doc)?;

    Ok(())
}

async fn download_and_append_to_comics(comic_number: u16, comics: &mut Vec<Comic>, client: &reqwest::Client) -> anyhow::Result<()> { // check if comics contains the comic, using iterators
    if !comics.iter().any(|com| com.num == comic_number) {
        let comic: Comic = client.get(format!("{URL}/{comic_number}/{INFO}")).send().await?.json().await?;

        println!("Adding comic number {comic_number}: \"{}\" to comics", comic.title);

        comics.push(comic);

        let file = File::create(DATA_PATH)?;
        serde_json::to_writer(file, &comics);
    } else {
        println!("Comic already exists in data");
    }

    Ok(())
}

async fn index_comic(c: &mut Comic, df: &mut ComicFrequency) -> anyhow::Result<ComicIndex> {
    let mut alt_frequencies: HashMap<String, usize> = HashMap::new();
    let mut title_frequencies: HashMap<String, usize> = HashMap::new();

    // index alt text
    let mut alt = c.alt.to_lowercase();
    alt.retain(|c| !r#"(),".;:--'"#.contains(c)); // strip punctuation

    let mut alen = 0;
    for word in alt.split_whitespace() {
        alen += 1;
        *alt_frequencies.entry(word.to_owned()).or_default() += 1;
    }

    let mut title = c.title.to_lowercase();
    title.retain(|c| !r#"(),".;:-'"#.contains(c)); // strip punctuation
    let mut tlen = 0;

    for word in title.split_whitespace() {
        *title_frequencies.entry(word.to_owned()).or_default() += 1;
        tlen += 1;
    }

    // for each unique word in alt text and title, add it to df

    for text in title_frequencies.keys() {
        *df.title_freq.entry(text.to_owned()).or_default() += 1;
    }

    for word in alt_frequencies.keys() {
        *df.alt_freq.entry(word.to_owned()).or_default() += 1;
    }

    let i = ComicIndex {
        title_freq: title_frequencies,
        title_len: tlen,
        alt_freq: alt_frequencies,
        alt_len: alen,
        comic: c.clone()
    };

    Ok(i)
}

fn search(q: &str, doc: &mut Document) -> Vec<(f32, ComicIndex)> {
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

// TODO: setup CLAP and parse args for downloading stuff
// TODO: start using transcripts for stuff as well
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();

    /*
    let mut latest: Comic = client.get(format!("{URL}/{INFO}")).send().await?.json().await?;
    download_all(latest, &client).await?;
    */
    
    let file = File::open(DATA_PATH)?;
    let mut doc: Document  = serde_json::from_reader(file)?;

    // download_and_append_to_comics(YOUR_COMIC_NUMBER as u16, &mut comics, &client).await?;
    
    let q = "google";

    let mut res = search(q, &mut doc);
    let top_20: Vec<(f32, ComicIndex)> = res.into_iter().take(20).collect();

    println!("Search query: {q}");
    for (rank, c) in top_20 {
        println!("#{} - {rank} - {}", c.comic.num, c.comic.title);
    }

    Ok(())
}
