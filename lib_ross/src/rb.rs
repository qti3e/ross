/// A queue using a ring buffer, it has a fixed capacity, writing more than
/// the capacity overwrites the previous data.
pub struct RingBuffer<'a, T> {
    data: &'a mut [Option<T>],
    w_index: usize,
    r_index: usize,
    capacity: usize,
}

impl<'a, T> RingBuffer<'a, T> {
    pub fn new(data: &'a mut [Option<T>]) -> Self {
        let capacity = data.len();
        Self {
            data,
            w_index: 0,
            r_index: 0,
            capacity,
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.data[self.r_index].is_none()
    }

    #[inline(always)]
    pub fn enqueue(&mut self, item: T) {
        self.data[self.w_index] = Some(item);
        self.w_index = (self.w_index + 1) % self.capacity;
    }

    #[inline(always)]
    pub fn dequeue(&mut self) -> Option<T> {
        let ret = self.data[self.r_index].take();
        if ret.is_some() {
            self.r_index = (self.r_index + 1) % self.capacity;
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use super::RingBuffer;

    #[test]
    fn test_full() {
        let mut q_slice = [None; 2];
        let mut q = RingBuffer::<i32>::new(&mut q_slice);
        q.enqueue(0);
        q.enqueue(1);
        assert_eq!(q.dequeue(), Some(0));
        assert_eq!(q.dequeue(), Some(1));
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.dequeue(), Some(2));
        assert_eq!(q.dequeue(), Some(3));
    }

    #[test]
    fn test_single() {
        let mut q_slice = [None; 2];
        let mut q = RingBuffer::<i32>::new(&mut q_slice);
        for i in 0..3 {
            q.enqueue(i);
            assert_eq!(q.dequeue(), Some(i));
        }
    }

    #[test]
    fn test_one_by_one() {
        let mut q_slice = [None; 2];
        let mut q = RingBuffer::<i32>::new(&mut q_slice);
        let mut items = Vec::new();
        q.enqueue(0);
        q.enqueue(1);
        while let Some(i) = q.dequeue() {
            items.push(i);
            if i <= 5 {
                q.enqueue(i + 2);
            }
        }
        assert_eq!(items, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_dequeue_empty() {
        let mut q_slice = [None; 2];
        let mut q = RingBuffer::<i32>::new(&mut q_slice);
        assert_eq!(q.dequeue(), None);
        q.enqueue(5);
        assert_eq!(q.dequeue(), Some(5));
    }
}
