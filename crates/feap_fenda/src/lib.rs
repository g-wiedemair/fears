//! Bindings to fenda

#![no_std]

unsafe extern "C" {
    fn foo();
    fn bar() -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binding() {
        unsafe {
            bar();
            foo();
        }
    }
}
