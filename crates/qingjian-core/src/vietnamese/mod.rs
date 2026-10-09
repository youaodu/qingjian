//! 越南语 Telex 词句查询。
//!
//! 这一套不复用中文拼音音节模型：Telex 输入先按越南语规则转写，再用独立词表和短语表查询。

mod dictionary;
mod query;
mod telex;

pub use dictionary::VietnameseDictionary;
pub use query::{VietnameseCandidate, VietnameseKind};
pub(crate) use telex::backspace_unit_len;
pub use telex::transcribe;
