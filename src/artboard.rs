use std::collections::HashMap;

use uuid::Uuid;

use crate::{BBox, Mandala, Path};

/// the artboard is responsible for holding,
/// incrementally rendering mandalas,
/// and other operations with them
pub struct Artboard {
    /// absolute bounds of the artboard
    bounds: BBox,
    /// all root level mandalas of the artboard
    roots: Vec<Mandala>,
    /// all rendered paths
    render: HashMap<Uuid, Vec<Path>>,
    /// order of layers
    layers: Vec<Uuid>,
}

impl Artboard {
    /// create new artboard with bounds
    pub fn new(bounds: BBox) -> Self {
        Self {
            bounds,
            roots: vec![],
            render: HashMap::new(),
            layers: vec![],
        }
    }

    /// add new mandala to the artboard and render it
    pub fn draw_mandala<F>(&mut self, draw_fn: &mut F)
    where
        F: FnMut(&BBox) -> Mandala,
    {
        let mndl = draw_fn(&self.bounds);
        let id = mndl.id;
        let exists = self.render.insert(id, mndl.render_paths());
        assert!(exists.is_none(), "mandala {id} is already drawn");
        self.layers.push(mndl.id);
        self.roots.push(mndl);
    }

    /// get all rendered paths
    pub fn view_paths(&self) -> Vec<Path> {
        self.layers
            .iter()
            .filter_map(|id| self.render.get(id))
            .flat_map(|p| p.iter().cloned())
            .collect()
    }

    pub fn update<'u, U>(&'u mut self, id: &'u Uuid) -> impl FnOnce(U) + 'u
    where
        U: FnMut(&mut Mandala),
    {
        |mut update| {
            if let Some(mndl) = self.roots.iter_mut().find(|m| m.id == *id) {
                update(mndl);
                self.render.insert(*id, mndl.render_paths());
            }
        }
    }
}
