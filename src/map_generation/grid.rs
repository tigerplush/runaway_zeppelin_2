use std::ops::{Index, IndexMut};

pub(super) struct Grid {
    contents: Vec<Option<usize>>,
    pub(super) width: usize,
    pub(super) height: usize,
}

impl Grid {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self {
            contents: vec![None; width * height],
            width,
            height,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Option<usize>;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let index = self.index(index.0, index.1);
        &self.contents[index]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let index = self.index(index.0, index.1);
        &mut self.contents[index]
    }
}

impl Index<usize> for Grid {
    type Output = Option<usize>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.contents[index]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.contents[index]
    }
}
