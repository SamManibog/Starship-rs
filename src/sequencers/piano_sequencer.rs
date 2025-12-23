use std::{cmp::Ordering, rc::{Rc, Weak}};

use crate::sequencers::note::{BeatUnits, Note};

/// a wrapper around a Weak<Note> that prevents any kind of promotion to an Rc
/// strong count is limited to 1
#[derive(Debug, Clone)]
pub struct NoteHandle(Weak<Note>);

impl NoteHandle {
    /// accesses the note immutably via a reader function
    /// to get mutable access to a note it must be removed from its pattern
    pub fn note(&self, reader: impl Fn(Option<&Note>)) {
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
    /// the note stored in the node
    key: OwnedNote,

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

impl PianoPattern {
    pub fn new() -> Self {
        Self {
            root: std::ptr::null_mut()
        }
    }

    /// removes the note from the tree, 
    /// returning the owned reference if found
    pub fn remove(&mut self, note: NoteHandle) -> Option<OwnedNote> {
        if !note.is_live() || self.root.is_null() {
            return None;
        }

        let note_ptr = note.0.as_ptr();

        // the path taken on the search for the note
        let mut ancestors = vec![self.root];

        // perform insertion and update path taken to insertion
        unsafe { loop {
            let node = *ancestors.last().unwrap();
            let ord = Self::note_cmp(note_ptr, Rc::as_ptr(&(*node).key.0));

            // the child to search next
            let child = if ord == Ordering::Less {
                &mut (*node).left
            } else if ord == Ordering::Greater {
                &mut (*node).right
            } else {
                // save stored note for later
                let output = OwnedNote((*node).key.0.clone());

                // pre-emptively remove the note from the list of ancestors
                // to avoid access-after-free error
                ancestors.pop();

                // the pointer to the removal target
                let parent_ptr = if let Some(p) = ancestors.last().copied() {
                    if (*p).left == node {
                        &mut (*p).left
                    } else {
                        &mut (*p).right
                    }
                } else {
                    &mut self.root
                };


                if !(*node).right.is_null() && !(*node).left.is_null(){
                    // handle case where we must find successor (2 children)

                    let mut successor_ancestors = Vec::new();

                    // find successor
                    let mut successor = (*node).right;
                    while !(*successor).left.is_null() {
                        successor_ancestors.push(successor);
                        successor = (*successor).left;
                    }
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

                Self::retract(self, ancestors);

                return Some(output);
            };

            if child.is_null() {
                return None;
            } else {
                ancestors.push(*child);
            }
        }}

    }

    /// inserts the note into the tree
    pub fn insert(&mut self, note: OwnedNote) {
        if self.root.is_null() {
            return;
        }

        let mut ancestors = vec![self.root];

        // perform insertion and update path taken to insertion
        loop {
            let node = unsafe { &mut **ancestors.last().unwrap() };
            let child = if Self::owned_note_cmp(&note, &node.key) == Ordering::Less {
                &mut node.left
            } else {
                &mut node.right
            };
            if child.is_null() {
                *child = Box::into_raw(Box::new(Node::new(note)));
                break;
            } else {
                ancestors.push(*child);
            }
        }

        // perform retracting
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

    fn owned_note_cmp(note1: &OwnedNote, note2: &OwnedNote) -> Ordering {
        unsafe { Self::note_cmp(Rc::as_ptr(&note1.0), Rc::as_ptr(&note2.0)) }
    }

    unsafe fn note_cmp(note1: *const Note, note2: *const Note) -> Ordering {
        unsafe {
            let start_ord = (*note1).start_time().cmp(&(*note2).start_time());
            if start_ord == Ordering::Equal {
                (*note1).end_time().cmp(&(*note2).end_time())
            } else {
                start_ord
            }
        }
    }

}

impl Node {
    /// creates a new node without children
    fn new(note: OwnedNote) -> Self {
        Self {
            max: note.note().end_time(),
            key: note,
            height: 0,
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
        }
    }
}
