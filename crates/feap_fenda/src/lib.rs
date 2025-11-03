//! Bindings to fenda

#![no_std]

unsafe extern "C" {
    pub fn foo();
    pub fn bar();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_binding() {
//         unsafe {
//             bar();
//             foo();
//         }
//     }
// }
