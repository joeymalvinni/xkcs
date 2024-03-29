use std::fs::File;
use futures::{stream, StreamExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use anyhow;
use reqwest;
use bincode;
use crate::utils::{DATA_PATH, URL, INFO};
use gxhash::GxHashMap;

const PARALLEL_REQUESTS: usize = 8;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comic {
    pub month: String,
    pub num: u16,
    pub year: String,
    pub day: String,
    pub title: String,
    pub alt: String,
    pub img: String,
    pub transcript: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComicIndex {
    pub title_freq: GxHashMap<String, usize>,
    pub title_len: usize,
    pub alt_freq: GxHashMap<String, usize>,
    pub alt_len: usize,
    pub comic: Comic,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComicFrequency {
    pub alt_freq: GxHashMap<String, usize>,
    pub title_freq: GxHashMap<String, usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    pub comics: Vec<ComicIndex>,
    pub frequency: ComicFrequency,
}

pub async fn download_all(latest: Comic, client: &reqwest::Client) -> anyhow::Result<()> {
    let c = Mutex::new(Vec::new());

    let frequencies = Arc::new(Mutex::new(ComicFrequency {
        title_freq: GxHashMap::default(),
        alt_freq: GxHashMap::default(),
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

    println!("Finished downloading and indexing.");
    let c = c.lock().await;
    let frequencies = frequencies.lock().await.clone();

    println!("Locked mutexes");
    let doc = Document {
        comics: (*c).to_vec(),
        frequency: frequencies,
    };

    println!("Creating file");
    let file = File::create(DATA_PATH)?;
    let _ = bincode::serialize_into(file, &doc)?;
    println!("Serialized files");

    Ok(())
}


pub async fn download_and_append_to_document(comic_number: u16, client: &reqwest::Client) -> anyhow::Result<()> { 
    let file = File::open(DATA_PATH)?;
    let reader = std::io::BufReader::new(file);
    let mut doc: Document = bincode::deserialize_from(reader)?;

    if !doc.comics.iter().any(|com| com.comic.num == comic_number) {
        let mut comic: Comic = client.get(format!("{URL}/{comic_number}/{INFO}")).send().await?.json().await?;

        let index: ComicIndex = index_comic(&mut comic, &mut doc.frequency).await?;

        doc.comics.push(index);

        let file = File::create(DATA_PATH)?;
        let _ = bincode::serialize_into(file, &doc)?;
    } else {
        println!("Comic already exists in data");
    }

    Ok(())
}

pub async fn index_comic(c: &mut Comic, df: &mut ComicFrequency) -> anyhow::Result<ComicIndex> {
    let mut alt_frequencies: GxHashMap<String, usize> = GxHashMap::default();
    let mut title_frequencies: GxHashMap<String, usize> = GxHashMap::default();

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
