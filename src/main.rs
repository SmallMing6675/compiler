#![allow(dead_code)] //Why does my IDE conplains about dead code
mod memory_allocator {
    pub mod chunk {
        use core::alloc::Layout;
        use std::alloc::alloc;
        use std::any::Any;

        pub enum ChunkError {
            BadSize, //requested size is not a multple of 4kb
            OutOfMemory,
            DoesNotExist, // requested memory is not in the heap
        }
        pub struct ChunkHeader {
            size: usize,
            avaliable: bool,
        }
        pub struct ChunkPointer<'a> {
            pub chunk: &'a mut Chunk<'a>,
        }

        pub struct Block<'a> {
            size: usize,
            inuse: bool,
            next: Option<&'a mut Block<'a>>,
        }
        pub struct Chunk<'a> {
            header: ChunkHeader,
            first_block: Option<&'a mut Block<'a>>,
        }

        pub fn init_chunk<'static>(chunk_size: usize) -> Result<Chunk<'static>, ChunkError> {
            let minimum_chunk_size = 4096;
            // the user must request a chunk size of a multiple of 4kb
            if chunk_size % minimum_chunk_size != 0 {
                return Err(ChunkError::BadSize);
            }

            let layout = Layout::new::<Chunk>();

            let chunk = Box::new(Chunk {
                header: ChunkHeader {
                    size: chunk_size,
                    avaliable: true,
                },
                first_block: None,
            });

            return Ok(*chunk);
        }

        impl Chunk<'_> {
            fn search<'a>(block: &'a mut Block<'a>) -> Option<&'a mut Block<'a>> {
                // Iterively search for a avaliable block to give back to the user
                let mut current = block;
                while ((&current).next).is_some() {
                    match (&current).next {
                        Some(t) => {
                            current = t;
                        }
                        None => return Some(&mut current),
                    }
                }

                return Some(&mut current);
            }

            pub fn allocate(self: &mut Self, size: usize) -> Result<&mut Block, ChunkError> {
                let layout = Layout::new::<Block>();

                let mut pointer = Block {
                    size: size,
                    inuse: true,
                    next: None,
                };

                match (self).first_block {
                    Some(b) => {
                        let last_block = Self::search(b).unwrap();
                        last_block.next = Some(&mut pointer);
                        return Ok(&mut pointer);
                    }

                    None => {
                        (*self).first_block = Some(&mut pointer);
                        return Ok(&mut pointer);
                    }
                }
            }

            pub fn free(self: &mut Self, pointer: &mut Block) -> Result<&mut Self, ChunkError> {
                fn get_block<'a>(
                    block: Option<&'a mut Block>,
                    pointer: &'a mut Block,
                ) -> Option<&'a mut Block<'a>> {
                    match block {
                        Some(b) => {
                            if *b == *pointer {
                                return Some(b);
                            }
                            get_block(block, pointer)
                        }

                        None => {
                            return None;
                        }
                    }
                }

                let block_to_free = get_block(self.first_block, pointer);

                match block_to_free {
                    Some(b) => {
                        b.inuse = false;
                        return Ok(self);
                    }

                    None => {
                        return Err(ChunkError::DoesNotExist);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn init_chunk() {
        let block = memory_allocator::chunk::init_chunk(4096);

        match block {
            Ok(b) => unsafe {
                println!("{}", b.chunk as usize);
                let chunk = b.chunk;

                let result = (*chunk).allocate(64);
                assert!(result.is_ok());

                for _ in 0..10 {
                    let result = (*chunk).allocate(32);
                    assert!(result.is_ok());
                }

                let result_two = (*chunk).allocate(128);
                match result_two {
                    Ok(t) => {
                        assert!((*chunk).free(t).is_ok());
                    }
                    Err(_) => {
                        panic!();
                    }
                }
            },
            Err(_) => {
                panic!();
            }
        }
    }
}

fn main() {}
