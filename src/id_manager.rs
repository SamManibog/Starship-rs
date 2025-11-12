use std::{cmp::Ordering, collections::VecDeque, marker::PhantomData};

#[derive(Debug)]
pub struct IdManager<T: From<u32> + Into<u32>> {
    pub min_id: u32,
    pub max_id: u32,

    /// the ranges of ids that are not in use (inclusive)
    /// Invariants: (let RMIN, RMAX be the minimum and maximum ids respectively)
    /// 	1) Ranges are non-overlapping
    /// 	2) Ranges are stored in sorted order by first value (trivially true if 1 and 3 hold)
    /// 	3) Ranges are stored in sorted order by last value (trivially true if 1 and 2 hold)
    /// 	4) For any range (a, b), a <= b
    /// 	5) There are no two ranges (a, b) and (b + 1, c)
    /// 	6) The for any range (a, b), RMIN <= a <= RMAX and RMIN <= b <= RMAX
    unused: VecDeque<(u32, u32)>,

    phantom: PhantomData<T>
}

impl<T: From<u32> + Into<u32>> IdManager<T> {
    pub fn new(min_id: u32, max_id: u32) -> Self {
        let mut unused = VecDeque::new();
        unused.push_front((min_id, max_id));
        Self {
            min_id,
            max_id,
            unused,
            phantom: PhantomData{}
        }
    }
}

impl<T: From<u32> + Into<u32>> Default for IdManager<T> {
    fn default() -> Self {
        Self::new(0, u32::MAX)
    }
}

impl<T: From<u32> + Into<u32>> IdManager<T> {
    /// Returns true if the current id is in use
    pub fn is_used(&self, id: T) -> bool {
        match self.search_used(id.into()) {
            Some(_) => true,
            _ => false
        }
    }

    /// Gets the id from the id manager
    pub fn get_id(&mut self) -> Option<T> {
        match self.extract_min() {
            Some(index) => Some(T::from(index)),
            None => None
        }
    }

    /// Removes the smallest id from the set of unused ids
    fn extract_min(&mut self) -> Option<u32> {
        if let Some(first) = self.unused.front_mut() {
            let out = first.0;
            if first.0 == first.1 {
                self.unused.pop_front();
            } else {
                first.0 += 1;
            }
            Some(out)
        } else {
            None
        }
    }

    /// Puts the given id back into the set of unused ids
    pub fn give_id(&mut self, id: T) {
        self.give_index(id.into());
    }

    /// Finds the index where unused[i].0 < id < unused[i].1
    /// If such an index exists (this will not be able to return Some(0) ever)
    fn search_unused(&self, id: u32) -> Option<usize> {
        let result = self.unused.binary_search_by(|(start, end)| {
            if *start > id {
                Ordering::Greater
            } else if *end < id {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        match result {
            Ok(i) => Some(i),
            Err(_) => None
        }
    }

    /// Finds the index i where unused[i - 1].1 < id < unused[i].0
    /// if such an index exists
    fn search_used(&self, id: u32) -> Option<usize> {
        let mut low = 1;
        let mut high = self.unused.len() - 1;
        let mut result = None;
        while low <= high {
            let mid = low + (high - low) / 2;
            if self.unused[mid - 1].1 < id && self.unused[mid].0 > id {
                result = Some(mid);
                break;
            } else if self.unused[mid - 1].1 > id {
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }
        result
    }

    /// Puts the given id back into the set of unused ids
    fn give_index(&mut self, id: u32) {
        debug_assert!(id >= self.min_id && id <= self.max_id, "Attempted to return out of bounds id to manager.");

        if self.unused.len() <= 0 {
            self.unused.push_front((id.clone(), id));
            return;
        }

        if self.unused[0].0 > id {
            // handle edge case where id is before unused
            if self.unused[0].0 == id + 1 {
                // absorb id into first range
                self.unused[0].0 -= 1;
            } else {
                // create new range with id
                self.unused.push_front((id.clone(), id));
            }
            return;
        }

        //perform binary search to find index at which to place id
        let index = self.search_used(id);

        if let Some(i) = index {
            if self.unused[i - 1].1 == id - 1 {
                // if the preceeding range is exactly before our id...
                if self.unused[i].0 == id + 1 {
                    // if the following range is exactly after our id,
                    // join ranges together
                    self.unused[i].0 = self.unused[i - 1].0;
                    self.unused.remove(i - 1);
                } else {
                    // otherwise,
                    // include id in the preceeding range
                    self.unused[i - 1].1 += 1;
                }

            } else if self.unused[i + 1].0 == id + 1 {
                // if the following  range is exactly after our id
                // include the id in the following range
                self.unused[i].0 -= 1;
            } else {
                // otherwise,
                // create new range
                self.unused.insert(i, (id.clone(), id));
            }
        }

    }

    /// Returns an iterator over all ids currently in use
    pub fn ids(&self) -> IdIterator<'_, T> {
        let (current_id, current_index) = if self.unused[0].0 == self.min_id {
            if self.unused.len() == 1 {
                (None, 0)
            } else {
                (Some(self.unused[0].1 + 1), 1)
            }
        } else {
            (Some(0), 0)
        };
        IdIterator {
            manager: &self,
            next_id: current_id,
            next_index: current_index
        }
    }
}

pub struct IdIterator<'a, T: From<u32> + Into<u32>> {
    /// the manager being iterated over
    manager: &'a IdManager<T>,

    /// the next id to output
    next_id: Option<u32>,

    /// the index of the next range of unused ids
    next_index: usize,
}

impl<T: From<u32> + Into<u32>> Iterator for IdIterator<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // determine output
        if self.next_id == None {
            return None;
        }
        let output_id = self.next_id.unwrap();

        // increment next id
        if self.next_index < self.manager.unused.len() {
            // if new index is within bounds
            if output_id < self.manager.unused[self.next_index].0 - 1 {
                // if new id is inside of range
                // increment id as normal
                self.next_id = Some(output_id + 1);
            } else {
                // otherwise,
                if self.manager.unused[self.next_index].1 == self.manager.max_id {
                    // if the end of the next range of unused ids contains the max id,
                    // there is no next index
                    self.next_id = None;
                } else {
                    // otherwise,
                    // 1) increase id using next index/range
                    self.next_id = Some(self.manager.unused[self.next_index].1 + 1);
                    // 2) increment next index
                    self.next_index += 1;
                }
            }
        } else {
            // if current_index is out of bounds
            if output_id < self.manager.max_id {
                // if next id is in bounds
                // increment it as normal
                self.next_id = Some(output_id + 1);

            } else {
                // otherwise,
                // set it to none
                self.next_id = None
            }
        }

        // return output id
        Some(T::from(output_id))
    }
}

