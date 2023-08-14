use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    Text,
    Image,
    Audio,
    Video,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationKind {
    Annotation,
    Trim,
    Crop,
    Resize,
    As(EntityKind),
    Summarize,
}

/// the identifier for all mio items including entities, operations and specters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MioId {
    /// the unique ord of the entity
    ord: usize,
    /// the millisecond timestamp of the entity's creation
    epoch: u64,
}

/// the moment of significant creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ephemerality {
    /// the timestamp of the moment
    pub time: SystemTime,
    /// the identifier of the entity that happens at that moment
    pub base: MioId,
}

impl PartialEq for Ephemerality {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}
impl Eq for Ephemerality {}

impl PartialOrd for Ephemerality {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
impl Ord for Ephemerality {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

/// the bases of the mio ring which contains all the raw data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// the identifier of the entity itself
    pub id: MioId,
    /// the kind of the entity
    pub kind: EntityKind,
    /// a pool of ord that can be assigned to the descendants of the entity
    pub pool: HashSet<usize>,
    /// the path to the resource of the entity in the file system
    pub path: PathBuf,
    /// the providence of the entity
    pub providence: Providence,
}

/// where the entity comes from, and how will it be treated
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum Providence {
    /// manually created
    #[default]
    Registered,
    /// generated during operation
    Induced,
    /// pinned by user
    Pinned,
}

/// the operation that can be done upon specters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// the identifier of the operation itself
    pub id: MioId,
    /// the kind of the operation
    pub kind: OperationKind,
    /// the attributes of the operation
    pub attr: Attr,
    /// the identifiers of the specters that the operation is based on
    pub base: Vec<MioId>,
    /// the identifier of the resulting specter
    pub res: MioId,
}

/// a dynamic polymorphic attribute data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Attr {
    Int(i32),
    Uint(u32),
    Float(f32),
    Str(String),
    Bool(bool),
    Array(Vec<Attr>),
    Map(HashMap<String, Attr>),
}

impl Operation {}

/// the generalized form of the entity which may represent either an entity or an operated entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specter {
    /// the identifier of the specter itself
    pub id: MioId,
    /// the identifier of the operation that results in the specter
    pub operation: MioId,
    pub cached: Option<PathBuf>,
}

impl Specter {
    pub fn actualize(self) {}
}

/// allocates new `MioId`s
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Alloc {
    /// next ord to be allocated, larger than all existing ords
    pub ord: usize,
    /// the co-pool of ords that can be re-allocated, collected from the freed items
    pub hill: HashSet<usize>,
}

impl Alloc {
    fn _allocate(&mut self) -> usize {
        if let Some(ord) = self.hill.iter().next().cloned() {
            self.hill.remove(&ord);
            ord
        } else {
            let ord = self.ord;
            self.ord += 1;
            ord
        }
    }
    pub fn allocate(&mut self) -> MioId {
        let ord = self._allocate();
        let epoch = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        MioId { ord, epoch }
    }
    pub fn allocate_pool(&mut self, size: usize) -> HashSet<usize> {
        let mut pool = HashSet::with_capacity(size);
        for _ in 0..size {
            pool.insert(self._allocate());
        }
        pool
    }
    pub fn deallocate(&mut self, id: MioId) {
        self.hill.insert(id.ord);
    }
    pub fn garbage_collection(&mut self) {
        while self.ord > 0 {
            let ord = self.ord - 1;
            if self.hill.contains(&ord) {
                self.hill.remove(&ord);
                self.ord = ord;
            } else {
                break;
            }
        }
    }
}

/// the distributed operations and resulting specters within the mio ring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MioDist {
    /// the operations within the mio ring
    pub operations: HashMap<MioId, Operation>,
    /// the specters within the mio ring
    pub specters: HashMap<MioId, Specter>,
}

impl MioDist {
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
            specters: HashMap::new(),
        }
    }
}

impl Default for MioDist {
    fn default() -> Self {
        Self::new()
    }
}

/// `MioDist` may seem like a monoid, but the RHS will overwrite the LHS if overlapped,
/// so `+=` is implemented instead of `+` to remind the user of the potential overwrite
impl std::ops::AddAssign for MioDist {
    fn add_assign(&mut self, other: Self) {
        self.operations.extend(other.operations);
        self.specters.extend(other.specters);
    }
}

/// each entity, once created, can be cached into a `MioThread` to concurrently operate on different entities;
/// note that the thread will stall if a specter from another thread is used during calculation, in which case
/// the common specter should be elevated into another `Entity`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MioThread {
    pub entity: Entity,
    pub cached: MioDist,
}

const POOL_SIZE: usize = 8;

/// the main data structure of the mio ring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mio {
    /// the allocator of `MioId`s
    pub alloc: Alloc,
    /// the null entity
    pub null: MioId,
    /// the ephemerality in chronological order
    pub chronology: Vec<Ephemerality>,
    /// the entities within the mio ring
    pub entities: HashMap<MioId, Entity>,
    /// the committed distributed operations
    pub committed: MioDist,
}

impl Mio {
    pub fn new() -> Self {
        let mut alloc = Alloc::default();
        let null = alloc.allocate();
        Self {
            alloc,
            null,
            chronology: Vec::new(),
            entities: HashMap::new(),
            committed: MioDist::new(),
        }
    }

    /// create a new mio thread while memorizing its entity into the mio ring
    pub fn register(&mut self, kind: EntityKind, path: PathBuf) -> MioThread {
        let id = self.alloc.allocate();
        let entity = Entity {
            id,
            kind,
            pool: self.alloc.allocate_pool(POOL_SIZE),
            path,
            providence: Providence::Registered,
        };
        self.chronology.push(Ephemerality {
            time: SystemTime::now(),
            base: id,
        });
        self.entities.insert(id, entity.clone());
        MioThread {
            entity,
            cached: MioDist::new(),
        }
    }

    pub fn commit(&mut self, thread: MioThread) {
        self.entities.insert(thread.entity.id, thread.entity).expect("entity not found");
        self.committed += thread.cached;
    }
}

impl Default for Mio {
    fn default() -> Self {
        Self::new()
    }
}

// pub struct Cached {}

// /// the operation that can be done upon specters
// pub trait Morphism {
//     /// the type of the source data needed, usually not dynamically polymorphic
//     type Source: Sized;
//     /// the type of the result, could be polymorphic or not
//     type Target;
//     /// validate the kind and existence of source and return it if valid
//     fn prepare(&self) -> Option<Self::Source>;
//     /// apply the morphism
//     fn execute(self, source: Self::Source) -> Self::Target;
// }
