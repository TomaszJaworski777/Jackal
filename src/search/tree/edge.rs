use spear::Move;

#[derive(Clone, Copy, PartialEq)]
pub struct Edge {
    node_index: i32,
    mv: Move,
    policy: i16,
    visits: u32,
    score: f32
}

impl Edge {
    pub fn new(node_index: i32, mv: Move, policy: f32) -> Self {
        Self {
            node_index,
            mv,
            policy: (policy * f32::from(i16::MAX)) as i16,
            visits: 0,
            score: 0.0
        }
    }
    
    pub fn index(&self) -> i32 {
        self.node_index
    }

    pub fn set_index(&mut self, index: i32) {
        self.node_index = index
    }

    pub fn mv(&self) -> Move {
        self.mv
    }

    pub fn policy(&self) -> f32 {
        f32::from(self.policy) / f32::from(i16::MAX)
    }

    pub fn visits(&self) -> u32 {
        self.visits
    }

    pub fn score(&self) -> f32 {
        self.score
    }

    pub fn avg_score(&self) -> f32 {
        if self.visits == 0 {
            0.5
        } else {
            self.score / self.visits as f32
        }
    }

    pub fn add_score(&mut self, score: f32) {
        self.visits += 1;
        self.score += score;
    }

    pub fn update_policy(&mut self, new_policy: f32) {
        self.policy = (new_policy * f32::from(i16::MAX)) as i16
    }
}