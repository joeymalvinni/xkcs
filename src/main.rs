mod comic;
mod search;
mod utils;

use comic::download_all;
use search::search;
use utils::Field;
use clap::Parser;
use std::fs::File;

#[derive(Parser, Debug)]
#[command(name = "XKCD Search Engine")]
#[command(author = "Joey M.")]
#[command(version = "1.0.0")]
#[command(about = "Simple search engine for XKCD comics.", long_about = None)]
struct Args {
    /// Query to search XKCD comics for.
    #[clap(short, long)]
    search: Option<String>,

    /// Limit the amount of search results. Defaults to 10.
    #[clap(short, long)]
    limit: Option<usize>,
}

// TODO: if a value is provided to --download, download [and index] that specific comic
// TODO: allow custom data path for downloading n stuff
// TODO: start using transcripts for stuff as well

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let mut limit = 10;
    let mut query = String::new();

    if let Some(l) = args.limit {
        limit = l;
    }

    if let Some(s) = args.search {
        query = s;
    }

    println!("{limit}");

    let client = reqwest::Client::new();

    /*
    let mut latest: Comic = client.get(format!("{URL}/{INFO}")).send().await?.json().await?;
    download_all(latest, &client).await?;
    */
    
    let file = File::open(utils::DATA_PATH)?;
    let mut doc: comic::Document  = serde_json::from_reader(file)?;

    // download_and_append_to_comics(YOUR_COMIC_NUMBER as u16, &mut comics, &client).await?;

    let mut res = search(&query, &mut doc);
    let top_20: Vec<(f32, comic::ComicIndex)> = res.into_iter().take(limit).collect();

    println!("Search query: {query}");
    for (rank, c) in top_20 {
        println!("#{} - {rank} - {}", c.comic.num, c.comic.title);
    }

    Ok(())
}
