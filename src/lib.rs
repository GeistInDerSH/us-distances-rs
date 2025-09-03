use farthest::Farthest;

pub struct Points {
    distances: Vec<Farthest>,
}

impl Points {
    #[inline]
    pub fn new(distances: Vec<Farthest>) -> Self {
        Self { distances }
    }

    pub fn farthest(&self) -> Farthest {
        *self
            .distances
            .iter()
            .max_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
            .unwrap_or(&Farthest::default())
    }
}

pub fn try_load_precalculated_distance_data(distance_data: &[u8]) -> Points {
    let kd: Vec<Farthest> =
        bincode::serde::decode_from_slice(distance_data, bincode::config::standard())
            .unwrap()
            .0;
    Points::new(kd)
}
