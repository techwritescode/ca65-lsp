use std::ops::{Index, IndexMut};

pub struct Arena<T>
where
    T: PartialEq,
{
    data: Vec<T>,
}

impl<T> Arena<T>
where
    T: PartialEq,
{
    pub fn new() -> Arena<T> {
        Arena { data: Vec::new() }
    }

    pub fn alloc(&mut self, item: T) -> usize {
        let idx = self.data.len();
        self.data.push(item);
        idx
    }
}

impl<T> Index<usize> for Arena<T>
where
    T: PartialEq,
{
    type Output = T;

    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for Arena<T>
where
    T: PartialEq,
{
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena() {
        let mut arena = Arena::new();
        let a = arena.alloc(1);
        let b = arena.alloc(2);
        let c = arena.alloc(3);
        let d = arena.alloc(4);

        assert_eq!(arena.data[a], 1);
        assert_eq!(arena.data[b], 2);
        assert_eq!(arena.data[c], 3);
        assert_eq!(arena.data[d], 4);
    }
}
