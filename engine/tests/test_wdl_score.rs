use engine::{AtomicWDLScore, WDLScore};

#[test]
fn reversed() { 
    assert_eq!(WDLScore::DRAW.reversed(), WDLScore::DRAW);
    assert_eq!(WDLScore::WIN.reversed(), WDLScore::LOSE);
    assert_eq!(WDLScore::LOSE.reversed(), WDLScore::WIN);
    assert!((0.2 - WDLScore::new(0.4, 0.4).reversed().win_chance()).abs() < 0.0001);
}

#[test]
fn lose_chance() { 
    assert_eq!(WDLScore::DRAW.lose_chance(), 0.0);
    assert_eq!(WDLScore::WIN.lose_chance(), 0.0);
    assert_eq!(WDLScore::LOSE.lose_chance(), 1.0);
    assert!(WDLScore::new(0.5, 0.5).lose_chance().abs() < 0.0001);
    assert!((0.2 - WDLScore::new(0.4, 0.4).lose_chance()).abs() < 0.0001);
}

#[test]
fn atomic() { 
    let atomic = AtomicWDLScore::default();

    assert_eq!(atomic.get_score_with_visits(0), WDLScore::new(0.0, 0.0));

    atomic.add(WDLScore::WIN);

    assert_eq!(atomic.get_score_with_visits(0), WDLScore::WIN);
    assert_eq!(atomic.get_score_with_visits(1), WDLScore::WIN);
    assert_eq!(atomic.get_score_with_visits(2), WDLScore::new(0.5, 0.0));

    atomic.add(WDLScore::LOSE);

    assert_eq!(atomic.get_score_with_visits(0), WDLScore::WIN);
    assert_eq!(atomic.get_score_with_visits(1), WDLScore::WIN);
    assert_eq!(atomic.get_score_with_visits(2), WDLScore::new(0.5, 0.0));

    atomic.add(WDLScore::DRAW);

    assert_eq!(atomic.get_score_with_visits(0), WDLScore::new(1.0, 1.0));
    assert_eq!(atomic.get_score_with_visits(1), WDLScore::new(1.0, 1.0));
    assert_eq!(atomic.get_score_with_visits(2), WDLScore::new(0.5, 0.5));
    assert_eq!(atomic.get_score_with_visits(4), WDLScore::new(0.25, 0.25));

    atomic.add(WDLScore::WIN);

    assert_eq!(atomic.get_score_with_visits(0), WDLScore::new(2.0, 1.0));
    assert_eq!(atomic.get_score_with_visits(1), WDLScore::new(2.0, 1.0));
    assert_eq!(atomic.get_score_with_visits(2), WDLScore::new(1.0, 0.5));
    assert_eq!(atomic.get_score_with_visits(4), WDLScore::new(0.5, 0.25));
}