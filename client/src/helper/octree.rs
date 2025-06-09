use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Clone)]
pub struct OctreeNode<T> {
    pub children: [Option<Box<OctreeNode<T>>>; 8],
    pub data: Vec<T>,
}

impl<T> Default for OctreeNode<T> {
    fn default() -> Self {
        Self {
            children: std::array::from_fn(|_| None),
            data: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Octree<T> {
    pub root: OctreeNode<T>,
}

impl<T> Octree<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new() -> Self {
        Self { root: OctreeNode::default() }
    }

    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let bytes = bincode::serialize(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, bytes)
    }

    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        let tree = bincode::deserialize(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(tree)
    }

    pub fn collect_data(&self, out: &mut VecDeque<T>) {
        self.root.collect_data(out);
    }
}

impl<T: Clone> OctreeNode<T> {
    fn collect_data(&self, out: &mut VecDeque<T>) {
        out.extend(self.data.clone());
        for child in self.children.iter().flatten() {
            child.collect_data(out);
        }
    }
}
