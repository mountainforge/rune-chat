use crate::Commit;

pub const GRAPH_COLORS: [u32; 6] = [0x9a5a00, 0x06748f, 0x6a3fc7, 0x23752f, 0xae2016, 0x2c55c2];
#[derive(Clone, Debug)]
pub struct GraphLine {
    pub from: usize,
    pub to: usize,
    pub half: u8,
    pub color: usize,
}
#[derive(Clone, Debug, Default)]
pub struct GraphRow {
    pub lane: usize,
    pub lines: Vec<GraphLine>,
}
impl GraphLine {
    pub fn points(&self, height: f32) -> Vec<[f32; 2]> {
        let x = |n: usize| 14.0 + n as f32 * 18.0;
        let (a, b) = match self.half {
            1 => (0.0, height / 2.0),
            2 => (height / 2.0, height),
            _ => (0.0, height),
        };
        let x1 = x(self.from);
        let x2 = x(self.to);
        (0..=12)
            .map(|i| {
                let t = i as f32 / 12.0;
                let u = 1.0 - t;
                [
                    u * u * u * x1 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x2,
                    u * u * u * a
                        + 3.0 * u * u * t * (a + (b - a) * 0.5)
                        + 3.0 * u * t * t * (b - (b - a) * 0.5)
                        + t * t * t * b,
                ]
            })
            .collect()
    }
}
/// Lanes track pending parent IDs, never branch-name guesses. Edges continue past a page boundary.
pub fn graph(commits: &[Commit]) -> (Vec<GraphRow>, usize) {
    let mut active: Vec<String> = vec![];
    let mut rows = vec![];
    let mut width = 1;
    for c in commits {
        let incoming = active.contains(&c.oid);
        if !incoming {
            active.push(c.oid.clone());
        }
        let lane = active.iter().position(|id| id == &c.oid).unwrap();
        let before = active.clone();
        active.remove(lane);
        for (i, p) in c.parents.iter().enumerate() {
            if !active.contains(p) {
                active.insert((lane + i).min(active.len()), p.clone());
            }
        }
        let mut lines = vec![];
        for (i, id) in before.iter().enumerate() {
            if id != &c.oid {
                let to = active.iter().position(|p| p == id).unwrap();
                lines.push(GraphLine {
                    from: i,
                    to,
                    half: 0,
                    color: i,
                });
            }
        }
        if incoming {
            lines.push(GraphLine {
                from: lane,
                to: lane,
                half: 1,
                color: lane,
            });
        }
        for p in &c.parents {
            let to = active.iter().position(|id| id == p).unwrap();
            lines.push(GraphLine {
                from: lane,
                to,
                half: 2,
                color: to,
            });
        }
        width = width.max(before.len()).max(active.len());
        rows.push(GraphRow { lane, lines });
    }
    (rows, width)
}
