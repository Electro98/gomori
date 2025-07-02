pub(crate) trait UniquePush<T: PartialEq> {
    fn push_unique(&mut self, val: T);
}

impl<T: Clone + PartialEq> UniquePush<&T> for Vec<T> {
    fn push_unique(&mut self, val: &T) {
        if !self.contains(val) {
            self.push(val.clone());
        }
    }
}

impl<T: PartialEq> UniquePush<T> for Vec<T> {
    fn push_unique(&mut self, val: T) {
        if !self.contains(&val) {
            self.push(val);
        }
    }
}
