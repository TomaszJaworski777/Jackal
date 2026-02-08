mod value_network;
mod policy_network;
mod layers;
mod inputs;

pub use crate::networks::value_network::ValueNetwork;
pub use crate::networks::policy_network::PolicyNetwork;

#[allow(non_upper_case_globals)]
pub static BaseValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/v40004096001q.network"))
};

#[allow(non_upper_case_globals)]
pub static Stage1ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/v40004096001qft3.network"))
};

#[allow(non_upper_case_globals)]
pub static Stage2ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/v40004096001qft5.network"))
};

#[allow(non_upper_case_globals)]
pub static BasePolicyNetwork: PolicyNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/p8008192009q.network"))
};

#[allow(non_upper_case_globals)]
pub static Stage1PolicyNetwork: PolicyNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/p1.network"))
};