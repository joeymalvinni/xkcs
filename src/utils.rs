pub const URL: &str = "https://xkcd.com";

pub const DATA_PATH: &str = "data/document";
pub const INFO: &str = "info.0.json";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const CYAN: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const RESET: &str = "\x1b[0m";

// TODO: use Transcript as a field and start indexing transcripts
pub enum Field {
    Title,
    Alt,
}
