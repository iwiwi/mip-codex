//! 未処理ノードの順序だけを変える。下界や枝刈り条件は変更しない。
use std::{cmp::Ordering, collections::BinaryHeap};
#[derive(Clone, Copy, Debug)]
pub enum Strategy {
    DepthFirst,
    BestBound,
    Hybrid,
}
struct Entry<T> {
    bound: f64,
    serial: usize,
    value: T,
}
impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.bound.total_cmp(&other.bound) == Ordering::Equal && self.serial == other.serial
    }
}
impl<T> Eq for Entry<T> {}
impl<T> PartialOrd for Entry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Entry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // 小さい最小化下界を先に。同じ下界なら最後に積んだ優先側を先に取る。
        other
            .bound
            .total_cmp(&self.bound)
            .then(self.serial.cmp(&other.serial))
    }
}
pub(crate) struct Queue<T> {
    strategy: Strategy,
    key: fn(&T) -> f64,
    stack: Vec<Entry<T>>,
    heap: BinaryHeap<Entry<T>>,
    next_serial: usize,
    pub pops: usize,
    pub best_pops: usize,
    pub transfers: usize,
    pub max_open: usize,
}
impl<T> Queue<T> {
    pub fn new(strategy: Strategy, key: fn(&T) -> f64) -> Self {
        Self {
            strategy,
            key,
            stack: Vec::new(),
            heap: BinaryHeap::new(),
            next_serial: 0,
            pops: 0,
            best_pops: 0,
            transfers: 0,
            max_open: 0,
        }
    }
    pub fn push(&mut self, value: T) {
        let bound = (self.key)(&value);
        assert!(!bound.is_nan());
        let entry = Entry {
            bound,
            serial: self.next_serial,
            value,
        };
        self.next_serial += 1;
        match self.strategy {
            Strategy::BestBound => self.heap.push(entry),
            Strategy::DepthFirst | Strategy::Hybrid => self.stack.push(entry),
        }
        self.max_open = self.max_open.max(self.stack.len() + self.heap.len());
    }
    pub fn pop(&mut self) -> Option<T> {
        if matches!(self.strategy, Strategy::Hybrid) && self.pops > 0 && self.pops % 64 == 0 {
            self.transfers += self.stack.len();
            self.heap.extend(self.stack.drain(..));
        }
        let next = if let Some(entry) = self.stack.pop() {
            Some(entry)
        } else {
            let result = self.heap.pop();
            if result.is_some() {
                self.best_pops += 1;
            }
            result
        };
        if next.is_some() {
            self.pops += 1;
        }
        next.map(|e| e.value)
    }
    pub fn min_bound(&self, initial: f64) -> f64 {
        self.stack
            .iter()
            .chain(self.heap.iter())
            .map(|e| e.bound)
            .fold(initial, f64::min)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn depth_first_keeps_last_child_preference() {
        let mut q = Queue::new(Strategy::DepthFirst, |v: &(usize, f64)| v.1);
        q.push((0, -10.));
        q.push((1, 5.));
        assert_eq!(q.min_bound(f64::INFINITY), -10.);
        assert_eq!(q.pop().unwrap().0, 1);
        assert_eq!(q.pop().unwrap().0, 0);
        assert!(q.pop().is_none());
    }
    #[test]
    fn best_bound_uses_minimum_and_deterministic_ties() {
        let mut q = Queue::new(Strategy::BestBound, |v: &(usize, f64)| v.1);
        for pair in [(0, -10.), (1, 5.), (2, -10.), (3, f64::NEG_INFINITY)] {
            q.push(pair);
        }
        for id in [3, 2, 0, 1] {
            assert_eq!(q.pop().unwrap().0, id);
        }
        assert!(q.pop().is_none());
    }
    #[test]
    fn hybrid_keeps_each_pending_node_exactly_once() {
        use std::collections::BTreeMap;
        let mut q = Queue::new(Strategy::Hybrid, |v: &(usize, f64)| v.1);
        let mut remaining = BTreeMap::new();
        for i in 0..400 {
            let bound = ((i * 73) % 401) as f64;
            remaining.insert(i, bound);
            q.push((i, bound));
        }
        let mut last_pops = 0;
        while !remaining.is_empty() {
            let expected = remaining.values().copied().fold(f64::INFINITY, f64::min);
            assert_eq!(q.min_bound(f64::INFINITY), expected);
            let (id, bound) = q.pop().unwrap();
            if last_pops > 0 && last_pops % 64 == 0 {
                assert_eq!(bound, expected);
            }
            assert_eq!(remaining.remove(&id), Some(bound));
            last_pops += 1;
        }
        assert!(q.pop().is_none());
        assert_eq!(q.pops, 400);
        assert_eq!(q.min_bound(123.), 123.);
        assert!(q.best_pops > 0 && q.transfers > 0);
    }
}

/// A constant-size conservative summary of regions whose LP infeasibility was not proved.
/// Individual regions are not retried or claimed pruned; the entire summary can be
/// removed only when its minimum inherited bound cannot improve the incumbent.
pub(crate) struct DeferredBounds {
    pub observed: usize,
    pub pending: usize,
    pub pruned: usize,
    pub minimum: f64,
}
impl Default for DeferredBounds {
    fn default() -> Self {
        Self {
            observed: 0,
            pending: 0,
            pruned: 0,
            minimum: f64::INFINITY,
        }
    }
}
impl DeferredBounds {
    pub fn record(&mut self, bound: f64) {
        assert!(!bound.is_nan());
        self.observed += 1;
        self.pending += 1;
        self.minimum = self.minimum.min(bound);
    }
    pub fn prune(&mut self, incumbent: f64) {
        if self.pending != 0 && incumbent.is_finite() && self.minimum >= incumbent {
            self.pruned += self.pending;
            self.pending = 0;
            self.minimum = f64::INFINITY;
        }
    }
}
#[cfg(test)]
mod deferred_tests {
    use super::DeferredBounds;
    #[test]
    fn unresolved_regions_hold_the_global_bound_until_all_are_prunable() {
        let mut d = DeferredBounds::default();
        d.record(-10.);
        d.record(-2.);
        d.prune(f64::INFINITY);
        assert_eq!(d.pending, 2);
        d.prune(-3.);
        assert_eq!(d.minimum, -10.);
        assert_eq!(d.pending, 2);
        d.prune(-10.);
        assert_eq!(d.pending, 0);
        assert_eq!(d.pruned, 2);
        assert_eq!(d.minimum, f64::INFINITY);
        d.record(-5.);
        assert_eq!(d.minimum, -5.);
        assert_eq!(d.observed, 3);
        d.prune(-6.);
        assert_eq!(d.pruned, 3);
        assert_eq!(d.pending, 0);
    }
    #[test]
    fn unbounded_inherited_lower_bound_cannot_be_dropped() {
        let mut d = DeferredBounds::default();
        d.record(f64::NEG_INFINITY);
        d.record(10.);
        d.prune(0.);
        assert_eq!(d.minimum, f64::NEG_INFINITY);
        assert_eq!(d.pending, 2);
    }
}
