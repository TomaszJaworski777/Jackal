mod value_network;
mod policy_network;
mod layers;
mod inputs;

use crate::networks::value_network::ValueNetwork;
use crate::networks::policy_network::PolicyNetwork;

#[allow(non_upper_case_globals)]
pub static ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/v40004096001q.network"))
};

#[allow(non_upper_case_globals)]
pub static PolicyNetwork: PolicyNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/p8008192009q.network"))
};