pub trait File: Send + Sync {
    fn read(&self, buf: UserBufer) -> usize;
    fn write(&self, buf: UserBufer) -> usize;
}
