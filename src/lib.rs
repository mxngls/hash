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
    V: Clone,
{
    const DEFAULT_SIZE: usize = 256;
    const RESIZE_THRESHOLD: f64 = 0.8;
    const RESIZE_FACTOR: usize = 2;

    pub fn new(hasher: H) -> Self {
        Self::with_capacity(Self::DEFAULT_SIZE, hasher)
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
        let hash = (self.hasher)(key);
        let mut index = (hash as usize) % self.capacity;

        while let Some(elem) = &self.buffer[index] {
            if elem.key == *key && !elem.removed {
                return index;
            }
            index = (index + 1) % self.capacity;
        }
        index
    }

    fn resize(&mut self) {
        let org_buffer = std::mem::take(&mut self.buffer);

        self.capacity *= Self::RESIZE_FACTOR;
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
        if self.len >= (self.capacity as f64 * Self::RESIZE_THRESHOLD) as usize {
            self.resize();
        }

        let elem = Elem {
            key,
            value,
            removed: false,
        };
        let slot = self.find_slot(&elem.key);

        if self.buffer[slot].is_none() {
            self.len += 1;
        }

        self.buffer[slot] = Some(elem);
    }

    pub fn get(&mut self, key: K) -> Option<V> {
        let slot = self.find_slot(&key);

        match &self.buffer[slot] {
            Some(elem) if !elem.removed => Some(elem.value.clone()),
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

#[cfg(test)]
mod test {
    use super::*;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hasher<T: Hash>(key: &T) -> u32 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as u32
    }

    #[test]
    fn test_default_creation() {
        let map: HashMap<String, String, fn(&String) -> u32> = HashMap::new(hasher);

        assert_eq!(map.capacity, 256);
    }

    #[test]
    fn test_with_capacity_creation() {
        let map: HashMap<String, String, fn(&String) -> u32> = HashMap::with_capacity(100, hasher);

        assert_eq!(map.capacity, 100);
    }

    #[test]
    fn test_insert() {
        let mut map = HashMap::new(hasher);

        map.insert("Hello,", "World");
    }

    #[test]
    fn test_get() {
        let mut map = HashMap::new(hasher);

        map.insert("Hello,", "World");
        assert_eq!("World", map.get("Hello,").unwrap());

        assert_eq!(None, map.get("Hi,"));
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::new(hasher);

        map.insert("Hello,", "World");
        map.remove("Hello,");

        assert_eq!(None, map.get("Hello,"))
    }

    #[test]
    fn test_resize() {
        let size = 10;
        let mut map = HashMap::with_capacity(size, hasher);

        for i in 0..size {
            map.insert(i, "number");
        }

        for i in 0..size {
            assert_eq!(Some("number"), map.get(i));
        }

        assert_eq!(size, map.len);
        assert_eq!(size * 2, map.capacity);
    }
}
