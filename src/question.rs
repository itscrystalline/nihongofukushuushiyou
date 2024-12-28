use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) struct Question {
    pub card_id: i32,
    pub front: String,
    pub front_image: Option<PathBuf>,
    pub correct_option: String,
    pub incorrect_options: Vec<String>
}

impl Question {}