
#[derive(Debug, Default)]
pub enum ViewState {
    #[default]
    Main,
    MinimumRequirements,
}

#[derive(Default)]
pub struct ViewPopup {
    pub active: bool,
    pub requirements: String,
    pub game_id: u32,
    pub current_game: usize,
    pub state: ViewState
}