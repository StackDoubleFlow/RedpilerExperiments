use serde::Deserialize;

pub type NodeId = usize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ComparatorMode {
    Compare,
    Subtract,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Repeater(u8),
    Comparator(ComparatorMode),
    Torch,
    StoneButton,
    StonePressurePlate,
    Lamp,
    Lever,
    Constant,
    Wire,
}

#[derive(Debug, Deserialize)]
pub enum LinkType {
    Default,
    Side,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub ty: LinkType,
    pub weight: u8,
    pub to: NodeId,
}

#[derive(Debug, Deserialize)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub ty: NodeType,
    pub inputs: Vec<Link>,
    pub updates: Vec<NodeId>,
    pub facing_diode: bool,
    pub comparator_far_input: Option<u8>,
    pub output_power: u8,
    /// Comparator powered / Repeater locked
    pub diode_state: bool,
    pub pos: BlockPos,
}
