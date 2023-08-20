use super::*;

pub struct MioView {
    pub timeline: Vec<Ephemerality>,
    pub ring: MioRing,
}

impl MioView {
    pub fn all(mio: &Mio) -> Self {
        MioViewGen::All.interpret(mio).unwrap()
    }
}

pub enum MioViewGen {
    All,
    Anchor {
        former: usize,
        anchor: usize,
        latter: usize,
    },
}

impl Interpretable for MioViewGen {
    type Mio<'a> = &'a Mio;
    type Target<'a> = MioView;
    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target<'a>> {
        let (low, high) = match self {
            MioViewGen::All => (0, mio.chronology.len() - 1),
            MioViewGen::Anchor {
                former,
                anchor,
                latter,
            } => (
                (anchor - former).clamp(0, mio.chronology.len() - 1),
                (anchor + latter).clamp(0, mio.chronology.len() - 1),
            ),
        };
        let timeline = if high < mio.chronology.len() {
            mio.chronology[low..=high].iter().cloned().collect_vec()
        } else {
            Vec::new()
        };
        let ring = MioRingGen {
            base: timeline.iter().map(|e| e.base).collect::<HashSet<_>>(),
        }
        .interpret(mio)?;
        Ok(MioView { timeline, ring })
    }
}

pub struct MioRingGen {
    pub base: HashSet<MioId>,
}

impl Interpretable for MioRingGen {
    type Mio<'a> = &'a Mio;
    type Target<'a> = MioRing;
    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target<'a>> {
        let mut ring = MioRing::new();

        let mut required = self.base;
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
        Ok(ring)
    }
}

pub struct MioInitiate {
    pub kind: OperationKind,
    pub attr: serde_json::Value,
    pub base: Vec<MioId>,
}

impl MioInitiate {
    pub fn new(attr: impl Operable, base: Vec<MioId>) -> Self {
        Self {
            kind: attr.kind(),
            attr: serde_json::to_value(attr).expect("failed to serialize attribute"),
            base,
        }
    }
}

impl Interpretable for MioInitiate {
    type Mio<'a> = &'a mut Mio;
    type Target<'a> = MioRing;
    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target<'a>> {
        let operation =
            {
                let mut allocator =
                    mio.specterish(self.base.iter().next().ok_or_else(|| {
                        anyhow::anyhow!("cannot initiate operation without base")
                    })?);

                let operation = allocator
                    .allocate()
                    .unwrap_or_else(|| mio.alloc.allocate())
                    .into();
                let specter = allocator
                    .allocate()
                    .unwrap_or_else(|| mio.alloc.allocate())
                    .into();
                
                allocator.deps_push(RingId::from(operation))?;
                allocator.ring(&mut mio.ring)?;

                let ext = self
                    .kind
                    .analyze(self.base.iter().map(|base| mio.specterish(base).kind()))
                    .unwrap()
                    .ext_hint();
                Specter {
                    id: specter,
                    ext,
                    nonce: Specter::<Lazy>::gen_nouce(),
                    deps: Vec::new(),
                    body: Lazy { operation },
                }
                .ring(&mut mio.ring)?;
                let operation = Operation {
                    id: operation,
                    kind: self.kind,
                    attr: self.attr,
                    base: self.base,
                    specter,
                };
                operation.ring_and(&mut mio.ring)?;
                operation
            };
        // return an incremental ring
        let mut ring = MioRing::new();
        for id in operation.base.iter().copied() {
            mio.specterish(&id).ring(&mut ring)?;
        }
        let specter = operation.specter;
        ring.specters
            .insert(specter, mio.ring.specters[&specter].clone());
        ring.operations.insert(operation.id, operation);
        Ok(ring)
    }
}

pub struct MioForce {
    pub ids: HashSet<MioId>,
}

impl Interpretable for MioForce {
    type Mio<'a> = &'a Mio;
    type Target<'a> = ();

    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target<'a>> {
        for id in self.ids {
            mio.specterish(&id).run(mio)?;
        }
        Ok(())
    }
}

pub enum MioArchive {
    Specter(MioId),
    Operation(OpId),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct MioArchived {
    pub mio_id: Vec<MioId>,
    pub op_id: Vec<OpId>,
}

impl AddAssign for MioArchived {
    fn add_assign(&mut self, rhs: Self) {
        self.mio_id.extend(rhs.mio_id);
        self.op_id.extend(rhs.op_id);
    }
}
impl AddAssign<MioId> for MioArchived {
    fn add_assign(&mut self, rhs: MioId) {
        self.mio_id.push(rhs);
    }
}
impl AddAssign<OpId> for MioArchived {
    fn add_assign(&mut self, rhs: OpId) {
        self.op_id.push(rhs);
    }
}

impl Interpretable for MioArchive {
    type Mio<'a> = &'a mut Mio;
    type Target<'a> = MioArchived;
    fn interpret<'a>(self, mio: Self::Mio<'a>) -> anyhow::Result<Self::Target<'a>> {
        let mut archived = MioArchived::default();
        match self {
            MioArchive::Specter(id) => {
                archived += id;
                let specter = mio.specterish(&id);
                specter.ring(&mut mio.archived)?;
                specter.unring(&mut mio.ring)?;
                mio.alloc.deallocate(id.into());
                mio.chronology.retain(|e| e.base != id);
                for dep in specter.deps().into_iter().map(Into::into) {
                    archived += MioArchive::Operation(dep).interpret(mio)?;
                }
            }
            MioArchive::Operation(id) => {
                archived += id;
                let operation = mio.ring.operations[&id].clone();
                operation.ring(&mut mio.archived)?;
                operation.unring(&mut mio.ring)?;
                mio.alloc.deallocate(id.into());
                archived += MioArchive::Specter(operation.specter).interpret(mio)?;
            }
        }
        Ok(archived)
    }
}
