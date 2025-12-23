use std::{cmp::Ordering, rc::{Rc, Weak}};

use crate::sequencers::note::{BeatUnits, Note};

/// a wrapper around a Weak<Note> that prevents any kind of promotion to an Rc
/// strong count is limited to 1
#[derive(Debug, Clone)]
pub struct NoteHandle(Weak<Note>);

impl NoteHandle {
    /// accesses the note immutably via a reader function
    /// to get mutable access to a note it must be removed from its pattern
    pub fn note<T>(&self, reader: impl Fn(Option<&Note>) -> T) -> T {
        if let Some(rc) = self.0.upgrade() {
            reader(Some(&rc))
        } else {
            reader(None)
        }
    }

    /// checks if the two handles are to the same note
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }

    /// checks if the owned note owns this handle
    pub fn is_handle_of(&self, owner: &OwnedNote) -> bool {
        self.0.as_ptr() == Rc::as_ptr(&owner.0)
    }

    /// checks how many other handles exist to the same note
    pub fn handle_count(&self) -> usize {
        self.0.weak_count()
    }

    /// checks if the note still exists
    pub fn is_live(&self) -> bool {
        self.0.strong_count() > 0
    }
}

/// a wrapper around Rc<Note> that prevents any kind of duplication of the note
#[derive(Debug)]
pub struct OwnedNote(Rc<Note>);

impl OwnedNote {
    /// creates a new instance of an owned note
    pub fn new(note: Note) -> Self {
        Self(Rc::new(note))
    }

    /// accesses the note immutably
    pub fn note(&self) -> &Note {
        &self.0
    }

    /// accesses the note mutably
    /// fails if there exist handles to this note
    pub fn note_mut(&mut self) -> Option<&mut Note> {
        Rc::get_mut(&mut self.0)
    }

    /// checks how many handles exist to the note
    pub fn handle_count(&self) -> usize {
        Rc::weak_count(&self.0)
    }

    /// checks if the handle points to this owned note
    pub fn is_owner_of(&self, handle: &NoteHandle) -> bool {
        Rc::as_ptr(&self.0) == handle.0.as_ptr()
    }
}

/// a pattern of notes data is stored as an augmented avl tree
pub struct PianoPattern {
    root: *mut Node
}

/// a node in the avl tree of a piano pattern
struct Node {
    /// the list of notes stored in the node
    /// notes that start and end at the same time are stored in the same node
    /// we guarantee that this vec is non-empty
    notes: Vec<OwnedNote>,

    /// the maximum end time found in self or either subtree
    max: BeatUnits,

    /// the height of the node
    /// its easier to implement an avl tree using height
    /// should also not affect size due to alignment of this struct being at least 4
    height: usize,

    /// the left child
    left: *mut Node,

    /// the right child
    right: *mut Node,
}

impl Node {
    fn key(&self) -> NodeKey {
        let note = self.notes[0].note();
        NodeKey::from_note(note)
    }

    fn start_time(&self) -> BeatUnits {
        self.key().0
    }
    
    fn end_time(&self) -> BeatUnits {
        self.key().1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeKey(BeatUnits, BeatUnits);

impl NodeKey {
    fn from_note(note: &Note) -> Self {
        Self(note.start_time(), note.end_time())
    }
}

impl PartialOrd for NodeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let start_ord = self.0.cmp(&other.0);
        if start_ord == Ordering::Equal {
            self.1.cmp(&other.1)
        } else {
            start_ord
        }
    }
}

impl PianoPattern {
    pub fn new() -> Self {
        Self {
            root: std::ptr::null_mut()
        }
    }

    /// queries the pattern for a list of notes occuring at the given time in beats
    pub fn query_time_inplace(&self, time: f64) -> Vec<NoteHandle> {
        let mut output = Vec::new();
        self.query_time(&mut output, time);
        output
    }

    /// queries the pattern for a list of notes occuring at the given in beats
    /// puts notes into the given vector
    pub fn query_time(&self, output: &mut Vec<NoteHandle>, time: f64) {
        self.query_range(output, time, time);
    }

    /// queries the pattern for a list of notes occuring within the given range in beats
    pub fn query_range_inplace(&self, start: f64, end: f64) -> Vec<NoteHandle> {
        let mut output = Vec::new();
        self.query_range(&mut output, start, end);
        output
    }

    /// queries the pattern for a list of notes occuring within the given range in beats
    /// puts notes into the given vector
    /// panics if start > end
    pub fn query_range(&self, output: &mut Vec<NoteHandle>, start: f64, end: f64) {
        assert!(start <= end, "Start must be less than or equal to end.");

        // the call stack of nodes to search
        let mut stack = Vec::new();

        if !self.root.is_null() && unsafe { (*self.root).max.into_beats() > start } {
            stack.push(self.root);
        }

        unsafe {
            while !stack.is_empty() {
                let node = &(*stack.pop().unwrap());
                let NodeKey(start_bu, end_bu) = node.key();
                let (node_start, node_end) = (start_bu.into_beats(), end_bu.into_beats());

                // add to output if necessary
                if node_start <= end && start <= node_end {
                    for note in &node.notes {
                        output.push(NoteHandle(Rc::downgrade(&note.0)));
                    }
                }

                // check which children we need to consider
                if !node.left.is_null() && start <= node_start && start <= (*node.left).max.into_beats() {
                    stack.push(node.left);
                }
                if !node.right.is_null() && node_start <= end && start <= (*node.right).max.into_beats() {
                    stack.push(node.right);
                }
            }
        }
    }

    /// removes the note from the tree, 
    /// returning the owned reference if found
    pub fn remove(&mut self, note: NoteHandle) -> Option<OwnedNote> {
        if !note.is_live() || self.root.is_null() {
            return None;
        }

        let note_key = note.note(|f| NodeKey::from_note(f.unwrap()) );

        // the path taken on the search for the note
        let mut ancestors = vec![self.root];

        // perform deletion and update path taken to deletion
        unsafe { loop {
            let node = *ancestors.last().unwrap();

            // the child to search next
            let child = match note_key.cmp(&(*node).key()) {
                Ordering::Less => &mut (*node).left,
                Ordering::Greater => &mut (*node).right,
                Ordering::Equal => {
                    // we found the node, break
                    break;
                }
            };

            if child.is_null() {
                return None;
            } else {
                ancestors.push(*child);
            }
        }}

        unsafe {
            // the node potentially containing the note
            let node = *ancestors.last().unwrap();

            // search and remove the note in the node
            let mut output = None;
            (*node).notes.retain(|n| {
                if n.is_owner_of(&note) {
                    output = Some(OwnedNote(n.0.clone()));
                    false
                } else {
                    true
                }
            });

            // we dont need to delete the node if it's non-empty: exit early
            if !(*node).notes.is_empty() {
                return output;
            }

            // pre-emptively remove the note from the list of ancestors
            // to avoid access-after-free error
            ancestors.pop();

            // the pointer within the tree to the node being removed
            let parent_ptr = if let Some(p) = ancestors.last().copied() {
                if (*p).left == node {
                    &mut (*p).left
                } else {
                    &mut (*p).right
                }
            } else {
                &mut self.root
            };


            // delete the node
            if !(*node).right.is_null() && !(*node).left.is_null(){
                // handle case where we must find successor (2 children)

                // path to the successor following the deleted note
                let mut successor_ancestors = Vec::new();

                // find successor
                let mut successor = (*node).right;
                while !(*successor).left.is_null() {
                    successor_ancestors.push(successor);
                    successor = (*successor).left;
                }
                // clean up/rebalance from new location
                ancestors.push(successor);

                // delete successor from parent
                if let Some(successor_parent) = successor_ancestors.last_mut().copied() {
                    (*successor_parent).left = (*successor).right;
                }

                // replace node with successor
                (*successor).left = (*node).left;
                *parent_ptr = successor;

                // moving successor may have caused imbalance, so fix it
                self.retract(successor_ancestors);

            } else {
                // handle cases where we don't need to look for successor
                // (no children, or 1 child)

                // remove from tree
                *parent_ptr = if (*node).right.is_null() {
                    (*node).left
                } else {
                        (*node).right
                    };
            }

            // drop node
            drop(Box::from_raw(node));

            // balance tree
            Self::retract(self, ancestors);

            // return the owned note
            output
        }
    }


    /// inserts the note into the tree
    pub fn insert(&mut self, note: OwnedNote) {
        if self.root.is_null() {
            return;
        }

        let note_key = NodeKey::from_note(note.note());
        let mut ancestors = vec![self.root];

        // perform insertion and update path taken to insertion
        loop {
            let node = unsafe { &mut **ancestors.last().unwrap() };
            let child = match note_key.cmp(&node.key()){
                Ordering::Less => &mut node.left,
                Ordering::Greater => &mut node.right,
                Ordering::Equal => {
                    node.notes.push(note);
                    return;
                }
            };

            if child.is_null() {
                *child = Box::into_raw(Box::new(Node::new(note)));
                break;
            } else {
                ancestors.push(*child);
            }
        }

        // rebalance
        unsafe { self.retract(ancestors) };
    }

    /// performs retracting on the given path from the root
    /// you must ensure that the path is valid and has no cycles
    unsafe fn retract(&mut self, mut path: Vec<*mut Node>) {
        while let Some(node) = path.pop() {
            unsafe {
                // the pointer to the node in the tree
                let node_ptr = if let Some(parent) = path.last() {
                    if node == (**parent).left {
                        &mut (**parent).left
                    } else {
                        &mut (**parent).right
                    }
                } else {
                    &mut self.root
                };

                Self::recalculate_max(node);
                Self::recalculate_height(node);

                let initial_height = (*node).height;
                *node_ptr = Self::rebalance(node);
                if initial_height == (**node_ptr).height {
                    return;
                }
            }
        }
    }

    /// rebalances at the node, returning the new root
    unsafe fn rebalance(node: *mut Node) -> *mut Node {
        unsafe {
            let bf = Self::balance_factor(node);
            if bf > 1 {
                if Self::balance_factor((*node).left) < 0 {
                    (*node).left = Self::rotate_left((*node).left);
                }
                Self::rotate_right(node)

            } else if bf < -1 {
                if Self::balance_factor((*node).right) < 0 {
                    (*node).right = Self::rotate_right((*node).right);
                }
                Self::rotate_left(node)

            } else {
                node
            }
        }
    }

    unsafe fn rotate_left(node: *mut Node) -> *mut Node {
        unsafe {
            let r_child = (*node).right;

            (*node).right = (*r_child).left;
            (*r_child).left = node;

            Self::recalculate_height(node);
            Self::recalculate_height(r_child);
            Self::recalculate_max(node);
            Self::recalculate_max(r_child);
            r_child
        }
    }

    unsafe fn rotate_right(node: *mut Node) -> *mut Node {
        unsafe {
            let l_child = (*node).left;

            (*node).left = (*l_child).right;
            (*l_child).right = node;

            Self::recalculate_height(node);
            Self::recalculate_height(l_child);
            Self::recalculate_max(node);
            Self::recalculate_max(l_child);
            l_child
        }
    }

    /// recalculates max at the given node only based on its children
    unsafe fn recalculate_max(node: *mut Node) {
        unsafe {
            let left_max = if (*node).left.is_null() {
                BeatUnits(0)
            } else {
                (*(*node).left).max
            };
            let right_max = if (*node).right.is_null() {
                BeatUnits(0)
            } else {
                (*(*node).right).max
            };
            (*node).max = left_max.max(right_max).max((*node).max);
        }
    }

    /// recalculates height at the given node only based on its children
    unsafe fn recalculate_height(node: *mut Node) {
        unsafe {
            let left_height = if (*node).left.is_null() {
                0
            } else {
                (*(*node).left).height + 1
            };
            let right_height = if (*node).right.is_null() {
                0
            } else {
                (*(*node).right).height + 1
            };
            (*node).height = left_height.max(right_height);
        }
    }

    unsafe fn balance_factor(node: *mut Node) -> i32 {
        unsafe {
            let left_height = if let Some(left) = (*node).left.as_mut() {
                left.height
            } else {
                0
            };
            let right_height = if let Some(right) = (*node).right.as_mut() {
                right.height
            } else {
                0
            };
            right_height as i32 - left_height as i32
        }
    }

}

impl Node {
    /// creates a new node without children
    fn new(note: OwnedNote) -> Self {
        Self {
            max: note.note().end_time(),
            notes: vec![note],
            height: 0,
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
        }
    }
}
