mod operation;
mod persistence;

use derive_more::From;
use directories_next::ProjectDirs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    Text,
    Image,
    Audio,
    Video,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationKind {
    Annotation,
    Trim,
    Crop,
    Resize,
    As(EntityKind),
    Summarize,
}

/// the underlying identifier for all mio ring items including entities, operations and specters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RingId {
    /// the millisecond timestamp of the entity's creation
    epoch: u64,
    /// the unique ord of the entity
    ord: usize,
}

/// the identifier for all entities and specters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From)]
pub struct MioId(RingId);
impl MioId {
    pub fn stem(&self) -> String {
        let MioId(RingId { epoch, ord }) = self;
        format!("{}-{}", epoch, ord)
    }
}

/// the identifier for all operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, From)]
pub struct OpId(RingId);

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

/// all specters have a kind
pub trait WithEntityKind {
    fn kind(&self) -> EntityKind;
}

/// all specters can be located, concrete or lazy alike
pub trait Locatable {
    fn locate(&self, dirs: &MioDirs) -> PathBuf;
    fn exists(&self, dirs: &MioDirs) -> bool {
        self.locate(dirs).exists()
    }
    fn write(&mut self, dirs: &MioDirs, content: &[u8]) -> anyhow::Result<()> {
        std::fs::write(self.locate(dirs), content)?;
        Ok(())
    }
    fn replace(&mut self, dirs: &MioDirs, alt: &Path) -> anyhow::Result<()> {
        std::fs::copy(alt, self.locate(dirs))?;
        Ok(())
    }
    // fn write_str(&self, dirs: &MioDirs, content: &str) -> anyhow::Result<()> {
    //     std::fs::write(self.locate(dirs), content)?;
    //     Ok(())
    // }
    fn remove(&mut self, dirs: &MioDirs) -> anyhow::Result<()> {
        std::fs::remove_file(self.locate(dirs))?;
        Ok(())
    }
}

/// all specters can be actualized, concrete or lazy alike;
/// it's just for the lazy ones, we need to also actualize the operation
pub trait Actualizable {
    fn run(&self, mio: &Mio) -> anyhow::Result<()>;
}

/// and all specters should be specterish
pub trait Specterish: WithEntityKind + Locatable + Actualizable {}
impl<T: WithEntityKind + Actualizable + Locatable> Specterish for T {}

/// the generalized form of the entity which may represent either a raw entity or an operated entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specter<Body: Actualizer> {
    /// the identifier of the specter itself
    pub id: MioId,
    /// the file extension, or "type", of the resulting entity
    pub ext: EntityExt,
    /// operations that directly depend on this specter
    pub deps: Vec<OpId>,
    /// the actualizer of the specter
    pub body: Body,
}
impl<Body: Actualizer> WithEntityKind for Specter<Body> {
    fn kind(&self) -> EntityKind {
        self.ext.kind()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityExt {
    Txt,
    Url,
    Png,
    Jpg,
    // Mp3,
    // Mp4,
}
impl WithEntityKind for EntityExt {
    fn kind(&self) -> EntityKind {
        match self {
            EntityExt::Txt => EntityKind::Text,
            EntityExt::Url => EntityKind::Text,
            EntityExt::Png => EntityKind::Image,
            EntityExt::Jpg => EntityKind::Image,
            // EntityExt::Mp3 => EntityKind::Audio,
            // EntityExt::Mp4 => EntityKind::Video,
        }
    }
}
impl Display for EntityExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityExt::Txt => write!(f, "txt"),
            EntityExt::Url => write!(f, "url"),
            EntityExt::Png => write!(f, "png"),
            EntityExt::Jpg => write!(f, "jpg"),
            // EntityExt::Mp3 => write!(f, "mp3"),
            // EntityExt::Mp4 => write!(f, "mp4"),
        }
    }
}

#[typetag::serde(tag = "actualizer")]
pub trait Actualizer {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concrete {
    /// a pool of ord that can be assigned to the descendants of the entity
    pub pool: HashSet<usize>,
    /// where the entity comes from, and how will it be treated
    pub providence: Providence,
}
#[typetag::serde]
impl Actualizer for Concrete {}

impl Locatable for Specter<Concrete> {
    fn locate(&self, dirs: &MioDirs) -> PathBuf {
        dirs.data_dir
            .join(format!("{}.{}", self.id.stem(), self.ext))
    }
}
impl Actualizable for Specter<Concrete> {
    /// since concrete specters are always valid, we don't need to do anything
    fn run(&self, _mio: &Mio) -> anyhow::Result<()> {
        Ok(())
    }
}

/// where the entity comes from, and how will it be treated
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum Providence {
    /// manually created
    #[default]
    Registered,
    /// elevated during operation
    Induced,
    /// pinned by user
    Pinned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lazy {
    /// the identifier of the operation that results in the specter
    pub operation: OpId,
}
#[typetag::serde]
impl Actualizer for Lazy {}

impl Locatable for Specter<Lazy> {
    fn locate(&self, dirs: &MioDirs) -> PathBuf {
        dirs.cache_dir
            .join(format!("{}.{}", self.id.stem(), self.ext))
    }
}
impl Actualizable for Specter<Lazy> {
    /// if the specter exists, do nothing; otherwise, run the operation
    fn run(&self, mio: &Mio) -> anyhow::Result<()> {
        if self.exists(&mio.dirs) {
            return Ok(());
        } else {
            mio.operations[&self.body.operation].run(mio)
        }
    }
}
impl Specter<Lazy> {
    /// elevate an actualized lazy specter to a concrete specter
    pub fn elevate(self, dirs: &MioDirs) -> anyhow::Result<Specter<Concrete>> {
        if self.exists(dirs) {
            let old_path = self.locate(dirs);
            let specter = Specter {
                id: self.id,
                ext: self.ext,
                deps: self.deps,
                body: Concrete {
                    pool: HashSet::new(),
                    providence: Providence::Induced,
                },
            };
            // move the file from cache to data
            std::fs::copy(old_path.as_path(), specter.locate(dirs))?;
            std::fs::remove_file(old_path.as_path())?;
            Ok(specter)
        } else {
            anyhow::bail!("specter not actualized")
        }
    }
}

/// the operation that can be done upon specters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// the identifier of the operation itself
    pub id: MioId,
    /// the kind of the operation
    pub kind: OperationKind,
    /// the attributes of the operation
    pub attr: serde_json::Value,
    /// the identifiers of the specters that the operation is based on
    pub base: Vec<MioId>,
    /// the identifier of the resulting specter
    pub specter: MioId,
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
    pub fn allocate(&mut self) -> RingId {
        let ord = self._allocate();
        let epoch = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        RingId { ord, epoch }
    }
    pub fn allocate_pool(&mut self, size: usize) -> HashSet<usize> {
        let mut pool = HashSet::with_capacity(size);
        for _ in 0..size {
            pool.insert(self._allocate());
        }
        pool
    }
    pub fn deallocate(&mut self, id: RingId) {
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

/// each entity, once created, can be cached into a `MioThread` to concurrently operate on different entities;
/// note that the thread will stall if a specter from another thread is used during calculation, in which case
/// the common specter should be elevated into another `Entity`
#[derive(Debug, Clone)]
pub struct MioThread {
    pub base: MioId,
    pub ops: Vec<OpId>,
}
impl MioThread {
    /// spawn a new mio thread
    pub fn new(base: MioId) -> Self {
        Self {
            base,
            ops: Vec::new(),
        }
    }
    fn add(&mut self, op: OpId) {
        self.ops.push(op);
    }
}

impl std::ops::AddAssign<OpId> for MioThread {
    /// append a new operation to the mio thread
    fn add_assign(&mut self, op: OpId) {
        self.add(op);
    }
}

/// path manager for mio ring which synthesizes new paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MioDirs {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl MioDirs {
    pub fn new() -> Self {
        let proj_dirs =
            ProjectDirs::from("", "LitiaEeloo", "MioRing").expect("failed to find project dirs");
        let config_dir = proj_dirs.config_dir().to_path_buf();
        let cache_dir = proj_dirs.cache_dir().to_path_buf();
        let data_dir = proj_dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(config_dir.as_path()).expect("failed to create config dir");
        std::fs::create_dir_all(cache_dir.as_path()).expect("failed to create cache dir");
        std::fs::create_dir_all(data_dir.as_path()).expect("failed to create data dir");
        Self {
            config_dir,
            cache_dir,
            data_dir,
        }
    }
}

impl Default for MioDirs {
    fn default() -> Self {
        Self::new()
    }
}

const POOL_SIZE: usize = 8;

/// the main data structure of the mio ring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mio {
    /// the path manager
    #[serde(skip)]
    pub dirs: MioDirs,
    /// the allocator of `MioId`s
    pub alloc: Alloc,
    /// the null entity
    pub null: MioId,
    /// the ephemerality in chronological order
    pub chronology: Vec<Ephemerality>,
    /// the entities within the mio ring
    pub entities: HashMap<MioId, Specter<Concrete>>,
    /// the operations within the mio ring
    pub operations: HashMap<OpId, Operation>,
    /// the specters within the mio ring
    pub specters: HashMap<MioId, Specter<Lazy>>,
}

impl Mio {
    pub fn new() -> Self {
        let mut alloc = Alloc::default();
        let null = alloc.allocate().into();
        Self {
            dirs: MioDirs::new(),
            alloc,
            null,
            chronology: Vec::new(),
            entities: HashMap::new(),
            operations: HashMap::new(),
            specters: HashMap::new(),
        }
    }

    /// create a new mio thread while memorizing its entity into the mio ring
    pub fn register(&mut self, persister: impl Persistable) -> anyhow::Result<()> {
        for (src, ext) in persister.persist()? {
            let id = self.alloc.allocate().into();
            let entity = Specter {
                id,
                ext,
                deps: Vec::new(),
                body: Concrete {
                    pool: self.alloc.allocate_pool(POOL_SIZE),
                    providence: Providence::Registered,
                },
            };
            self.chronology.push(Ephemerality {
                time: SystemTime::now(),
                base: id,
            });
            self.entities
                .entry(id)
                .and_modify(|e| unreachable!("duplicate entry found when registering {:?}", e))
                .or_insert(entity)
                .replace(&self.dirs, src.as_path())?
        }
        Ok(())
    }

    pub fn specterish(&self, id: &MioId) -> Box<dyn Specterish> {
        if let Some(entity) = self.entities.get(id) {
            Box::new(entity.clone())
        } else if let Some(specter) = self.specters.get(id) {
            Box::new(specter.clone())
        } else {
            unreachable!("specter not found")
        }
    }
}

impl Default for Mio {
    fn default() -> Self {
        Self::new()
    }
}

// pub struct Cached {}

/// the persistable can be persisted into the file system
pub trait Persistable {
    fn persist(&self) -> anyhow::Result<Vec<(PathBuf, EntityExt)>>;
}

/// the operable can be done upon specters
pub trait Operable: Sized + for<'de> Deserialize<'de> {
    type Source<'a>;
    type Target<'a>;

    /// validate the kind and existence of source and return it if valid
    fn prepare(op: &Operation) -> anyhow::Result<Self> {
        let value = serde_json::from_value(op.attr.clone())?;
        Ok(value)
    }
    /// apply the operator
    fn execute<'a>(self, src: Self::Source<'a>, tar: Self::Target<'a>) -> anyhow::Result<()>;
}
