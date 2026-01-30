// UI Features Module
pub mod themes;
pub mod reader;
pub mod search;
pub mod spell_checker;

pub use themes::ThemeManager;
pub use reader::ReaderMode;
pub use search::SearchEngine;
pub use spell_checker::SpellChecker;