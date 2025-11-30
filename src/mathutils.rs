pub fn u32_increment_wrap(x: u32, min: u32, max: u32) -> u32 {
    if x == max { min } else { x + 1 }
}

pub fn u32_decrement_wrap(x: u32, min: u32, max: u32) -> u32 {
    dbg!((x, min, max));
    if x == min { max } else { x - 1 }
}
