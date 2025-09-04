use farthest::{encoding_config, Farthest};

pub struct Points {
    distances: Vec<Farthest>,
}

impl Points {
    #[inline]
    pub fn new(distances: Vec<Farthest>) -> Self {
        Self { distances }
    }

    pub fn farthest(&self) -> Farthest {
        let mut max = *self.distances.first().unwrap_or(&Farthest::default());
        for farthest in self.distances.iter().skip(1) {
            if farthest.distance > max.distance {
                max = *farthest;
            }
        }
        max
    }
}

pub fn try_load_precalculated_distance_data(distance_data: &[u8]) -> Points {
    let kd: Vec<Farthest> = bincode::serde::decode_from_slice(distance_data, encoding_config())
        .unwrap()
        .0;
    Points::new(kd)
}
