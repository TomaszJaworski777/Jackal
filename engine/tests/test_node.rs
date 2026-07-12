use engine::Node;

#[test]
fn node_size() {
    assert_eq!(std::mem::size_of::<Node>(), 48);
}
