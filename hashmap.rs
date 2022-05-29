use std::alloc::{alloc, dealloc, Layout};

const DEF_CAP: usize = 3;

struct Node<V: Clone> {
    key: usize,
    value: V,
    is_added: bool,
}

struct Bucket<V: Clone> {
    capacity: usize,
    size: usize,
    nodes: *mut Node<V>,
}

pub struct Hashmap<V: Clone> {
    capacity: usize,
    buckets: *mut Bucket<V>,
}

impl<V: Clone> Hashmap<V> {
    pub unsafe fn new_hashmap() -> Hashmap<V> {
        let capacity = crate::hashmap::make_prime(DEF_CAP) as usize;
        let mut bucket_count = 0 as usize;
        let layout_nodes_buffer = Layout::array::<*mut Node<V>>(capacity).unwrap();
        let layout_buckets_buffer = Layout::array::<*mut Bucket<V>>(capacity).unwrap();
        let mut bucket_buffer_ptr = alloc(layout_buckets_buffer) as *mut Bucket<V>;
        let bucket_base_ptr = bucket_buffer_ptr;
        while bucket_count < DEF_CAP {
            let mut node_count = 0 as usize;
            let mut node_ptr = alloc(layout_nodes_buffer) as *mut Node<V>;
            let node_base_ptr = node_ptr;
            while node_count < DEF_CAP {
                (*node_ptr).is_added = false;
                node_count = node_count + 1;
                node_ptr = node_ptr.add(1);
            }
            (*bucket_buffer_ptr).nodes = node_base_ptr;
            (*bucket_buffer_ptr).size = 0;
            (*bucket_buffer_ptr).capacity = capacity;
            bucket_count = bucket_count + 1;
            bucket_buffer_ptr = bucket_buffer_ptr.add(1);
        }
        Hashmap {
            capacity,
            buckets: bucket_base_ptr,
        }
    }

    unsafe fn get_hash(&self, key: usize) -> usize {
        return (key as u32 % self.capacity as u32) as usize;
    }

    pub unsafe fn add_item(&self, key: usize, value: V) -> bool {
        let mut index = self.get_hash(key) as usize;
        let mut bucket = self.buckets.add(index) as *mut Bucket<V>;
        if (*bucket).size < (*bucket).capacity {
            index = 0;
            while index < (*bucket).capacity {
                let mut node = (*bucket).nodes.add(index) as *mut Node<V>;
                if (*node).is_added == false
                // check its empty
                {
                    (*node).key = key;
                    (*node).value = value;
                    (*node).is_added = true;
                    (*bucket).size = (*bucket).size + 1;
                    return true;
                }
                if (*node).key == key
                // update value, associate with key
                {
                    (*node).value = value;
                    return true;
                }
                index = index + 1;
            }
        }
        // capacity excess, extend array and return here!
        else {
            self.extend_array(index);
            return self.add_item(key, value);
        }
        false
    }

    pub unsafe fn remove_item(&self, key: usize) -> bool {
        let mut index = self.get_hash(key) as usize;
        let mut bucket_ptr = self.buckets.add(index) as *mut Bucket<V>;
        index = 0;
        while index < (*bucket_ptr).capacity {
            if (*(*bucket_ptr).nodes.add(index)).key == key {
                (*(*bucket_ptr).nodes.add(index)).is_added = false;
                (*bucket_ptr).size = (*bucket_ptr).size - 1;
                // no need to freed, we'll use this node again.
                return true;
            }
            index = index + 1;
        }
        false
    }

    pub unsafe fn peek_item(&self, key: usize) -> Option<&V> {
        let mut index = self.get_hash(key) as usize;
        let bucket_ptr = self.buckets.add(index) as *mut Bucket<V>;
        index = 0;
        while index < (*bucket_ptr).capacity {
            if (*(*bucket_ptr).nodes.add(index)).key == key {
                return Some(&(*(*bucket_ptr).nodes.add(index)).value);
            }
            index = index + 1;
        }
        None
    }

    unsafe fn extend_array(&self, mut index: usize) -> () {
        let mut bucket_ptr = self.buckets.add(index) as *mut Bucket<V>;
        let old_cap = (*bucket_ptr).capacity as usize;
        let base_old_node_ptr = (*bucket_ptr).nodes;
        (*bucket_ptr).capacity = crate::hashmap::make_prime(old_cap * 2);
        let layout_nodes_buffer = Layout::array::<*mut Node<V>>((*bucket_ptr).capacity).unwrap();
        let mut new_node = alloc(layout_nodes_buffer) as *mut Node<V>;
        let base_node_ptr = new_node;
        index = 0;
        while index < (*bucket_ptr).capacity {
            (*new_node).is_added = false;
            if index < old_cap {
                let old_node = (*bucket_ptr).nodes.add(index) as *mut Node<V>;
                (*new_node).key = (*old_node).key;
                (*new_node).value = (*old_node).value.clone();
                (*new_node).is_added = (*old_node).is_added;
            }
            index = index + 1;
            new_node = new_node.add(1);
        }
        (*bucket_ptr).nodes = base_node_ptr;
        dealloc(
            base_old_node_ptr as *mut u8,
            Layout::array::<*mut Node<V>>(old_cap).unwrap(),
        );
    }
}

fn make_prime(mut number: usize) -> usize {
    if number <= 2 {
        return 2;
    }
    let mut count = 2 as usize;
    loop {
        if number % count == 0 {
            number = number + 1;
            return make_prime(number);
        }
        count = count + 1;
        if count == number {
            return number;
        }
    }
}
