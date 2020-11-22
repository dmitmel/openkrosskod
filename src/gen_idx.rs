// Based on <https://github.com/fitzgen/generational-arena/blob/7758d1d8ef65cadf7d7db2ef6d9086c8547d8b55/src/lib.rs>
// See also:
// <https://www.youtube.com/watch?v=aKLntZcp27M>
// <https://www.reddit.com/r/rust/comments/99hujc/generationalarena_a_safe_arena_using_generational/>
// <http://bitsquid.blogspot.com/2014/08/building-data-oriented-entity-system.html>
// <https://gist.github.com/jaburns/ca72487198832f6203e831133ffdfff4>

// impl Index {
//   pub fn from_raw_parts(index: usize, generation: Generation) -> Self {
//     Self { index, generation }
//   }

//   pub fn index(self) -> usize { self.index }
//   pub fn generation(self) -> Generation { self.generation }
// }

// #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
// pub struct GenIdx {
//   index: usize,
//   generation: Generation,
// }

// impl GenIdx {
//   pub fn index(&self) -> usize { self.index }
//   pub fn generation(&self) -> Generation { self.generation }
// }

// struct GenIdxAllocatorEntry {
//   is_alive: bool,
//   generation: Generation,
// }

// pub struct GenIdxAllocator {
//   entries: Vec<GenIdxAllocatorEntry>,
//   free: Vec<usize>,
// }

// impl GenIdxAllocator {
//   pub fn alloc(&mut self) -> GenIdx { todo!() }
//   pub fn free(&mut self, gen_idx: GenIdx) -> bool { todo!() }
//   pub fn is_alive(&self, gen_idx: GenIdx) -> bool { todo!() }
// }

// struct GenIdxVecEntry<T> {
//   value: T,
//   generation: Generation,
// }

// pub struct GenIdxVec<T>(Vec<Option<GenIdxVecEntry<T>>>);

// impl<T> GenIdxVec<T> {
//   // can update a value with an older generation
//   pub fn set(&mut self, gen_idx: GenIdx, value: T) { todo!() }

//   // generation must match
//   pub fn get(&self, gen_idx: GenIdx) -> Option<&T> { todo!() }
//   pub fn get_mut(&mut self, gen_idx: GenIdx) -> Option<&T> { todo!() }
// }
