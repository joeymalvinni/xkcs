mod comic;
mod search;
mod utils;

use comic::{download_all, download_and_append_to_document};
use search::search;
use utils::{URL, INFO, RED, RESET, GREEN, CYAN, MAGENTA};
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

    /// Download and index a specific comic.
    #[clap(short, long)]
    download: Option<usize>,

    /// Download all comics.
    #[clap(short = 'D', long)]
    download_all: bool, 
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
        let mut doc: comic::Document  = serde_json::from_reader(file)?;

        if query.is_empty() {
            println!("Search query is required.");
            return Ok(());
        }

        let res = search(&query, &mut doc);
        let top_20: Vec<(f32, comic::ComicIndex)> = res.into_iter().take(limit).collect();

        let padding = 4;
        let max_rank_len = 6 + padding;
        let max_name_len = top_20.iter().map(|(_, c)| c.comic.title.len()).max().unwrap_or(0).max(6) + padding;
        let max_num_len = top_20.iter().map(|(_, c)| c.comic.num.to_string().len()).max().unwrap_or(0).max(13) + padding;

        println!("┌{0:─<width_rank$}┬{0:─<width_rank$}┬{0:─<width_name$}┬{0:─<width_num$}┐", "", width_rank = max_rank_len, width_name = max_name_len, width_num = max_num_len);
        println!("│{MAGENTA} {:<width_rank$}{RESET}│{CYAN} {:<width_rank$}{RESET}│{GREEN} {:<width_name$}{RESET}│{RED} {:<width_num$}{RESET}│", "Index", "Rank", "Title", "Comic Number", width_rank = max_rank_len-1, width_name = max_name_len-1, width_num = max_num_len-1);
        println!("├{0:─<width_rank$}┼{0:─<width_rank$}┼{0:─<width_name$}┼{0:─<width_num$}┤", "", width_rank = max_rank_len, width_name = max_name_len, width_num = max_num_len);

        for (index, (rank, c)) in top_20.iter().enumerate() {
            println!(
                "│ {:<width_rank$}│ {:<width_rank$.4}│ {:<width_name$}│ {:<width_num$}│",
                index + 1,
                -rank,
                c.comic.title,
                c.comic.num,
                width_rank = max_rank_len-1,
                width_name = max_name_len-1,
                width_num = max_num_len-1,
            );
        }

        println!("└{0:─<width_rank$}┴{0:─<width_rank$}┴{0:─<width_name$}┴{0:─<width_num$}┘", "", width_rank = max_rank_len, width_name = max_name_len, width_num = max_num_len);
    }



    Ok(())
}
