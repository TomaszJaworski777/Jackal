use engine::SearchEngine;

#[test]
fn bench() { 
    let mut search_engine = SearchEngine::new();
    let (result, _) = search_engine.bench(Some(3));
    assert_ne!(result, 0);

    let (result, _) = search_engine.bench(Some(3));
    assert_ne!(result, 0);

    let (result, _) = search_engine.bench(Some(3));
    assert_ne!(result, 0);
}