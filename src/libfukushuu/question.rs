use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct Question {
    pub card_id: i32,
    pub front: (Option<String>, Option<PathBuf>),
    pub correct_option: (Option<String>, Option<PathBuf>),
    pub incorrect_options: Vec<(Option<String>, Option<PathBuf>)>
}

impl Question {}