use eva::data;

#[data(copy, ord)]
pub enum GameFileFormat {
    BinaryData,
}

#[data]
pub struct GameFileInfo {
    pub format: GameFileFormat,
}
