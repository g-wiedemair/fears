use feap_fenda::{database::DataBase, project::Project};

#[repr(C)]
enum ControlParameter {
    Initialize = 0,
}

/// FENDA cli rebuild
fn main() {
    // // Globals for now
    // // Length of data memory
    // let mtot: u32 = 1;
    // // Maximum number of dynamic data buffer
    // let maxcor: u32 = 9_100_000;

    // Initialize data base
    let db = DataBase::new();

    // Initialize project
    // name=Project, na=0, nr=256, nc=0
    let project = Project::new();
    
    println!("TODO");

    // unsafe {
    //     let input = ControlParameter::Initialize;
    //     fmacro(&(input as i32));
    // }
}
