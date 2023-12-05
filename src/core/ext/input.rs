pub type SearchQuery = String;
pub type Source = u64;
pub type Destination = i64;
pub type ItemUuid = String;
pub type Locale = String;

pub enum Command {
    Search(SearchQuery),
    GetLink(ItemUuid),
    Download(ItemUuid),
    Help,
    Ignore
}

pub trait Input {
    fn get_command(&self) -> Command;
    fn get_source(&self) -> Source;
    fn get_destination(&self) -> Destination;
    fn get_locale(&self) -> Locale;
}
