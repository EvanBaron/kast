struct PoolEntry<T> {
    value: Option<T>,
    generation: u32,
}

pub struct Pool<T> {
    entries: Vec<PoolEntry<T>>,
    free_indices: Vec<u32>,
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> (u32, u32) {
        if let Some(index) = self.free_indices.pop() {
            let entry = &mut self.entries[index as usize];
            entry.value = Some(value);
            entry.generation += 1;
            (index, entry.generation)
        } else {
            let index = self.entries.len() as u32;
            self.entries.push(PoolEntry {
                value: Some(value),
                generation: 1,
            });

            (index, 1)
        }
    }

    pub fn get(&self, index: u32, generation: u32) -> Option<&T> {
        self.entries.get(index as usize).and_then(|entry| {
            if entry.generation == generation {
                entry.value.as_ref()
            } else {
                None
            }
        })
    }

    pub fn get_mut(&mut self, index: u32, generation: u32) -> Option<&mut T> {
        self.entries.get_mut(index as usize).and_then(|entry| {
            if entry.generation == generation {
                entry.value.as_mut()
            } else {
                None
            }
        })
    }

    pub fn remove(&mut self, index: u32, generation: u32) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(index as usize) {
            if entry.generation == generation && entry.value.is_some() {
                self.free_indices.push(index);
                return entry.value.take();
            }
        }

        None
    }

    pub fn drain(&mut self) -> impl Iterator<Item = (u32, T)> {
        self.entries
            .drain(..)
            .enumerate()
            .filter_map(|(index, entry)| entry.value.map(|value| (index as u32, value)))
    }
}
