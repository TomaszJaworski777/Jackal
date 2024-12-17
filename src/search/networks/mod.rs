mod accumulator;
mod layer;
mod policy;
mod value;

pub(super) use accumulator::Accumulator;
pub(super) use layer::NetworkLayer;
pub use policy::PolicyNetwork;
pub use value::ValueNetwork;

pub const QA: i16 = 255;
pub const QB: i16 = 64;