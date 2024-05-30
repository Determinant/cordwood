use shale::{MemStore, MummyItem, ObjPtr, ObjRef, ShaleError, ShaleStore};

use std::fmt::Debug;
use std::io::{Cursor, Read, Write};

#[derive(Debug)]
pub enum ArrayError {
    Shale(ShaleError),
    Format(std::io::Error),
}

pub enum Node {
    Root(ObjPtr<Node>),
    Array(Vec<u64>),
}

impl Node {
    const ROOT: u8 = 0x0;
    const ARRAY: u8 = 0x1;

    fn as_root(&self) -> Option<ObjPtr<Node>> {
        match self {
            Self::Root(ptr) => Some(*ptr),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<u64>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<u64>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }
}

impl MummyItem for Node {
    fn hydrate(addr: u64, mem: &dyn MemStore) -> Result<Self, ShaleError> {
        let dec_err = |_| ShaleError::DecodeError;
        const META_SIZE: u64 = 1 + 8; // # of items in the array
        let meta_raw = mem.get_view(addr, META_SIZE).ok_or(ShaleError::LinearMemStoreError)?;
        Ok(match meta_raw[0] {
            Self::ROOT => {
                let ptr = u64::from_le_bytes(meta_raw[1..META_SIZE as usize].try_into().unwrap());
                Self::Root(unsafe { ObjPtr::new_from_addr(ptr) })
            }
            _ => {
                let n = u64::from_le_bytes(meta_raw[1..META_SIZE as usize].try_into().map_err(dec_err)?);
                let array_raw = mem
                    .get_view(addr + META_SIZE, n * 8)
                    .ok_or(ShaleError::LinearMemStoreError)?;
                let mut array = Vec::new();

                let mut cur = Cursor::new(array_raw.deref());
                let mut buff = [0; 8];
                for _ in 0..n {
                    cur.read_exact(&mut buff).map_err(|_| ShaleError::DecodeError)?;
                    array.push(u64::from_le_bytes(buff));
                }
                Self::Array(array)
            }
        })
    }

    fn dehydrated_len(&self) -> u64 {
        match self {
            Self::Root(_) => 1 + 8,
            Self::Array(a) => 1 + 8 + a.len() as u64 * 8,
        }
    }

    fn dehydrate(&self, to: &mut [u8]) {
        let mut cur = Cursor::new(to);
        match self {
            Self::Root(ptr) => {
                cur.write_all(&[Self::ROOT]).unwrap();
                cur.write_all(&ptr.addr().to_le_bytes()).unwrap();
            }
            Self::Array(a) => {
                cur.write_all(&[Self::ARRAY]).unwrap();
                cur.write_all(&(a.len() as u64).to_le_bytes()).unwrap();
                for v in a.iter() {
                    cur.write_all(&v.to_le_bytes()).unwrap();
                }
            }
        }
    }
}

pub struct Array {
    store: Box<dyn ShaleStore<Node>>,
}

impl Array {
    fn get_node(&self, ptr: ObjPtr<Node>) -> Result<ObjRef<Node>, ArrayError> {
        self.store.get_item(ptr).map_err(ArrayError::Shale)
    }
    fn new_node(&self, item: Node) -> Result<ObjRef<Node>, ArrayError> {
        self.store.put_item(item, 0).map_err(ArrayError::Shale)
    }
    fn free_node(&mut self, ptr: ObjPtr<Node>) -> Result<(), ArrayError> {
        self.store.free_item(ptr).map_err(ArrayError::Shale)
    }
}

impl Array {
    pub fn new(store: Box<dyn ShaleStore<Node>>) -> Self {
        Self { store }
    }

    pub fn init(root: &mut ObjPtr<Node>, store: &dyn ShaleStore<Node>) -> Result<(), ArrayError> {
        Ok(*root = store
            .put_item(
                Node::Root(
                    store
                        .put_item(Node::Array(Vec::new()), 0)
                        .map_err(ArrayError::Shale)?
                        .as_ptr(),
                ),
                0,
            )
            .map_err(ArrayError::Shale)?
            .as_ptr())
    }

    pub fn get_store(&self) -> &dyn ShaleStore<Node> {
        self.store.as_ref()
    }

    pub fn dump(&self, root: ObjPtr<Node>, w: &mut dyn Write) -> Result<(), ArrayError> {
        let array_ref = self.get_node(self.get_node(root)?.as_root().unwrap())?;
        let array = array_ref.as_array().unwrap();
        write!(
            w,
            "[{}]\n",
            array.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
        )
        .map_err(ArrayError::Format)
    }

    pub fn set(&mut self, idx: u64, value: u64, root: ObjPtr<Node>) -> Result<(), ArrayError> {
        let mut root_ref = self.get_node(root)?;
        let array_ptr = root_ref.as_root().unwrap();
        let array_node = self.get_node(array_ptr)?;
        let array = array_node.as_array().unwrap();
        let idx = idx as usize;
        if idx < array.len() {
            self.get_node(array_ptr)?
                .write(|a| a.as_array_mut().unwrap()[idx] = value);
        } else {
            let mut a = array.clone();
            a.resize(idx + 1, 0);
            a[idx] = value;
            let new_array = self.new_node(Node::Array(a))?;
            root_ref.write(|r| *r = Node::Root(new_array.as_ptr()));
            self.free_node(array_ptr)?;
        }
        Ok(())
    }

    pub fn get(&self, idx: u64, root: ObjPtr<Node>) -> Result<u64, ArrayError> {
        let array_ref = self.get_node(self.get_node(root)?.as_root().unwrap())?;
        let array = array_ref.as_array().unwrap();
        Ok(array[idx as usize])
    }

    pub fn flush_dirty(&self) -> Option<()> {
        self.store.flush_dirty()
    }
}
