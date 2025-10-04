pub trait BitboardTrait {
    fn new() -> Self;
    fn fold_and(&self) -> u64;
    fn fold_or(&self) -> u64;
    fn fold_xor(&self) -> u64;
}