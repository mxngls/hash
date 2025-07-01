use std::mem;

struct Elem<K, V> {
    key: K,
    value: V,
    psl: u8,
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
    K: PartialEq + Clone,
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

    /// Returns index for insertion/lookup. May point to:
    ///     1. Empty slot - key absent
    ///     2. Matching key - key found
    ///     3. Wrong key - PSL exceeded, key absent
    /// Callers must verify key equality for case 3.
    fn find_elem(&mut self, key: &K) -> usize {
        let hash = (self.hasher)(key);
        let mut psl = 0;
        let mut index = (hash as usize) % self.capacity;

        loop {
            match &mut self.buffer[index] {
                None => return index,
                Some(elem) if psl > elem.psl || elem.key == *key => {
                    return index;
                }
                _ => {
                    index = (index + 1) % self.capacity;
                    psl += 1;
                }
            }
        }
    }

    fn resize(&mut self) {
        let org_buffer = std::mem::take(&mut self.buffer);

        self.capacity *= Self::RESIZE_FACTOR;
        self.buffer = (0..self.capacity).map(|_| None).collect();
        self.len = 0;

        for elem in org_buffer.into_iter().flatten() {
            let index = self.find_elem(&elem.key);
            self.buffer[index] = Some(elem);
            self.len += 1;
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.len >= (self.capacity as f64 * Self::RESIZE_THRESHOLD) as usize {
            self.resize();
        }

        let hash = (self.hasher)(&key);
        let mut new = Elem { key, value, psl: 0 };

        let mut index = (hash as usize) % self.capacity;

        while let Some(existing) = &mut self.buffer[index] {
            // overwrite existing
            if existing.key == new.key {
                existing.value = new.value;
                return;
            }

            // swap
            if new.psl > existing.psl {
                mem::swap(&mut new, existing);
            }

            index = (index + 1) % self.capacity;
            new.psl += 1;
        }

        // insert new
        self.buffer[index] = Some(new);
        self.len += 1;
    }

    pub fn get(&mut self, key: K) -> Option<V> {
        let index = self.find_elem(&key);

        match &self.buffer[index] {
            Some(elem) if elem.key == key => Some(elem.value.clone()),
            _ => None,
        }
    }

    pub fn remove(&mut self, key: K) {
        let mut index = self.find_elem(&key);

        match &mut self.buffer[index] {
            Some(elem) if elem.key == key => {
                // remove element
                self.buffer[index] = None;

                index = (index + 1) % self.capacity;

                // backward shift elements belonging to current bucket
                while let Some(elem) = &mut self.buffer[index] {
                    if elem.psl == 0 {
                        break;
                    }

                    elem.psl -= 1;
                    self.buffer[index] = self.buffer[index - 1].take();
                    index = (index + 1) % self.capacity;
                }
            }
            _ => {}
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
        let mut map = HashMap::with_capacity(16, hasher);

        map.insert("Hello,", "World");
    }

    #[test]
    fn test_insert_overwrite() {
        let mut map = HashMap::with_capacity(16, hasher);

        map.insert("Hello,", "World");
        map.insert("Hello,", "Me");

        assert_eq!("Me", map.get("Hello,").unwrap());
        assert_eq!(1, map.len);
        assert!(
            map.buffer
                .iter()
                .filter_map(|elem| elem.as_ref())
                .all(|elem| elem.value == "Me")
        );
    }

    #[test]
    fn test_insert_overwrite_removed() {
        let mut map = HashMap::with_capacity(16, hasher);

        map.insert("Hello,", "World");
        map.remove("Hello,");
        map.insert("Hello,", "Me");

        assert_eq!("Me", map.get("Hello,").unwrap());
        assert_eq!(1, map.len);
        assert!(
            map.buffer
                .iter()
                .filter_map(|elem| elem.as_ref())
                .all(|elem| elem.value == "Me")
        );
    }

    #[test]
    fn test_get() {
        let mut map = HashMap::with_capacity(16, hasher);

        map.insert("Hello,", "World");

        assert_eq!("World", map.get("Hello,").unwrap());
        assert_eq!(None, map.get("Hi,"));
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::with_capacity(16, hasher);

        map.insert("Hello,", "World");
        map.remove("Hello,");

        assert_eq!(None, map.get("Hello,"));
        assert!(
            !map.buffer
                .iter()
                .filter_map(|elem| elem.as_ref())
                .any(|elem| elem.key == "Hello,")
        );
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

        assert_eq!(size * 2, map.buffer.len());
        assert_eq!(size * 2, map.capacity);
        assert_eq!(size, map.len);
        assert_eq!(map.capacity, map.buffer.len(),);
    }
}
