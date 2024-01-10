mod comic;
mod search;
mod utils;
mod table;

use comic::{download_all, download_and_append_to_document};
use search::search;
use utils::{URL, INFO};
use clap::Parser;
use std::fs::File;
use bincode;

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

    /// Download and index a specific comic.
    #[clap(short, long)]
    download: Option<usize>,

    /// Download all comics.
    #[clap(short = 'D', long)]
    download_all: bool, 
}

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

    let client = reqwest::Client::new();

    if args.download_all {
        let latest: comic::Comic = client.get(format!("{URL}/{INFO}")).send().await?.json().await?;

        println!("Downloading and indexing comics 1..={}.", latest.num);

        download_all(latest, &client).await?;
    } else if let Some(c) = args.download {
        println!("Downloading comic number {c}");
        download_and_append_to_document(c as u16, &client).await?;
    } else {
        let file = File::open(utils::DATA_PATH)?;
        let reader = std::io::BufReader::new(file);
        let mut doc: comic::Document = bincode::deserialize_from(reader)?;

        if query.is_empty() {
            // enter interactive mode
            let _ = search::interactive_mode(&mut doc);
        } else {
            let res = search(&query, &mut doc);
            let top_20: Vec<(f32, comic::ComicIndex)> = res.into_iter().take(limit).collect();
            
            table::print_table(top_20);
        }
    }

    Ok(())
}
