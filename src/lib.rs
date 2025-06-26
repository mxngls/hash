struct Elem<K, V> {
    key: K,
    value: V,
    removed: bool,
}

pub struct HashMap<K, V, H> {
    buffer: Vec<Option<Elem<K, V>>>,
    capacity: usize,
    hasher: H,
    len: usize,
}

impl<K, V, H> HashMap<K, V, H>
where
    H: Fn(&K) -> u32,
    K: PartialEq,
{
    const INITIAL_SIZE: usize = 256;
    const RESIZE_THRESHOLD_NUM: usize = 4;
    const RESIZE_THRESHOLD_DEM: usize = 5;
    const RESIZE_FACTOR: usize = 2;

    pub fn new(hasher: H) -> Self {
        Self::with_capacity(Self::INITIAL_SIZE, hasher)
    }

    pub fn with_capacity(capacity: usize, hasher: H) -> Self {
        Self {
            buffer: (0..capacity).map(|_| None).collect(),
            capacity,
            hasher,
            len: 0,
        }
    }

    fn find_slot(&mut self, key: &K) -> usize {
        let hash = (self.hasher)(&key);
        let mut index = (hash as usize) % self.buffer.len();

        while let Some(elem) = &self.buffer[index] {
            if elem.key == *key && !elem.removed {
                return index;
            }
            index = (index + 1) % self.buffer.len();
        }
        index
    }

    fn resize(&mut self) {
        let org_buffer = std::mem::replace(&mut self.buffer, Vec::new());

        self.capacity = self.capacity * Self::RESIZE_FACTOR;
        self.buffer = (0..self.capacity).map(|_| None).collect();
        self.len = 0;

        for elem in org_buffer
            .into_iter()
            .flatten()
            .filter(|elem| !elem.removed)
        {
            let slot = self.find_slot(&elem.key);
            self.buffer[slot] = Some(elem);
            self.len += 1;
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.len >= self.buffer.len() * (Self::RESIZE_THRESHOLD_NUM / Self::RESIZE_THRESHOLD_DEM)
        {
            self.resize();
        }

        let elem = Elem {
            key,
            value,
            removed: false,
        };
        let slot = self.find_slot(&elem.key);

        if self.buffer[slot].is_none() {
            self.len = self.len + 1;
        }

        self.buffer[slot] = Some(elem);
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
        let slot = self.find_slot(&key);

        match &self.buffer[slot] {
            Some(elem) if !elem.removed => Some(&elem.value),
            _ => None,
        }
    }

    pub fn remove(&mut self, key: K) {
        let slot = self.find_slot(&key);

        if let Some(elem) = &mut self.buffer[slot] {
            if elem.removed {
                return;
            }
            elem.removed = true;
            self.len -= 1;
        }
    }
}
