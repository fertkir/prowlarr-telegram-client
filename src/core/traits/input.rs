pub type SearchQuery = Box<str>;
pub type Source = u64;
pub type Destination = i64;
pub type ItemUuid = Box<str>;
pub type Locale = Box<str>;

pub enum Command {
    Search(SearchQuery),
    GetLink(ItemUuid),
    Download(ItemUuid),
    Help
}

pub trait Input: Send + Sync {
    fn get_command(&self) -> Command;
    fn get_source(&self) -> Source;
    fn get_destination(&self) -> Destination;
    fn get_locale(&self) -> Locale;
}
