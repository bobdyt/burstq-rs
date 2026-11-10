use rand;

pub fn guid() -> String {
    format!("{:08x}", rand::random::<u32>())
}
