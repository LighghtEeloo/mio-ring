use super::*;

impl Mio {
    /// initiate an operation
    fn initiate(&mut self, kind: OperationKind) -> anyhow::Result<&mut Operation> {
        let id = self.alloc.allocate().into();
        let specter = self.alloc.allocate().into();
        let operation = Operation {
            id,
            kind,
            attr: serde_json::Value::Null,
            base: Vec::new(),
            specter,
        };
        Ok(self
            .ring
            .operations
            .entry(id)
            .and_modify(|e| unreachable!("duplicate entry found when initiating {:?}", e))
            .or_insert(operation))
    }
}

pub struct MioView {
    pub view: Vec<Ephemerality>,
    pub ring: MioRing,
}

pub struct MioViewAnchor {
    pub former: usize,
    pub anchor: usize,
    pub latter: usize,
}

impl Interpretable for MioViewAnchor {
    type Mio<'a> = &'a Mio;
    type Target = MioView;
    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target> {
        let low = (self.anchor - self.former).clamp(0, mio.chronology.len() - 1);
        let high = (self.anchor + self.latter).clamp(0, mio.chronology.len() - 1);
        let view = mio.chronology[low..=high].iter().cloned().collect_vec();
        let mut ring = MioRing::new();

        let mut required = view.iter().map(|e| e.base).collect::<HashSet<_>>();
        let mut done = HashSet::new();

        while required.difference(&done).count() > 0 {
            let todo = required.difference(&done).copied().collect::<HashSet<_>>();
            let mut add_to_map = |id| -> HashSet<_> {
                done.insert(id);
                let deps = if let Some(entity) = mio.ring.entities.get(&id) {
                    ring.entities.insert(id, entity.clone());
                    &entity.deps
                } else if let Some(specter) = mio.ring.specters.get(&id) {
                    ring.specters.insert(id, specter.clone());
                    &specter.deps
                } else {
                    unreachable!("specter not found")
                };
                deps.iter().copied().collect()
            };
            let mut ops = HashSet::new();
            for id in todo {
                ops.extend(add_to_map(id));
            }
            for op in ops {
                let operation = &mio.ring.operations[&op];
                ring.operations.insert(op, operation.clone());
                for id in operation.base.iter().copied() {
                    // don't trace indirect entities
                    add_to_map(id);
                }
                required.insert(operation.specter);
            }
        }
        Ok(MioView { view, ring })
    }
}

pub struct MioInitiate {
    pub kind: OperationKind,
    pub attr: serde_json::Value,
    pub base: Vec<MioId>,
}

impl Interpretable for MioInitiate {
    type Mio<'a> = &'a mut Mio;
    type Target = MioRing;
    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target> {
        let operation = {
            let operation = mio.initiate(self.kind)?;
            operation.attr = self.attr;
            operation.base = self.base;
            operation.clone()
        };
        // return an incremental ring
        let mut ring = MioRing::new();
        for entity in operation.base.iter().copied() {
            ring.entities
                .insert(entity, mio.ring.entities[&entity].clone());
        }
        let specter = operation.specter;
        ring.specters
            .insert(specter, mio.ring.specters[&specter].clone());
        ring.operations.insert(operation.id, operation);
        Ok(ring)
    }
}

pub enum MioDelete {
    Specter(MioId),
    Operation(OpId),
}

#[derive(Default)]
pub struct MioDeleted {
    pub mio_id: Vec<MioId>,
    pub op_id: Vec<OpId>,
}

impl AddAssign for MioDeleted {
    fn add_assign(&mut self, rhs: Self) {
        self.mio_id.extend(rhs.mio_id);
        self.op_id.extend(rhs.op_id);
    }
}
impl AddAssign<MioId> for MioDeleted {
    fn add_assign(&mut self, rhs: MioId) {
        self.mio_id.push(rhs);
    }
}
impl AddAssign<OpId> for MioDeleted {
    fn add_assign(&mut self, rhs: OpId) {
        self.op_id.push(rhs);
    }
}

impl Interpretable for MioDelete {
    type Mio<'a> = &'a mut Mio;
    type Target = MioDeleted;
    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target> {
        let mut deleted = MioDeleted::default();
        match self {
            MioDelete::Specter(id) => {
                deleted += id;
                let specter = mio
                    .ring
                    .entities
                    .remove(&id)
                    .ok_or_else(|| anyhow::anyhow!("specter not found"))?;
                mio.alloc.deallocate(id.into());
                mio.chronology.retain(|e| e.base != id);
                for dep in specter.deps {
                    deleted += MioDelete::Operation(dep).interpret(mio)?;
                }
            }
            MioDelete::Operation(id) => {
                deleted += id;
                let operation = mio
                    .ring
                    .operations
                    .remove(&id)
                    .ok_or_else(|| anyhow::anyhow!("operation not found"))?;
                mio.alloc.deallocate(id.into());
                deleted += MioDelete::Specter(operation.specter).interpret(mio)?;
            }
        }
        Ok(deleted)
    }
}
