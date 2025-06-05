# Measuring Kani's Performance

- both the performance of the kani compiler itself and the driver as a whole. 
    - because of how the compiler is called by the driver, they must be done separately 
- set env var and it will be outputted 
- samply must be installed
- for the compiler, you must make sure to cargo clean so it actually recompiles
    - what it exactly looks like can depend on the build cache?, so it's helpful to right click on `kani_compiler::codegen_cprover_gotoc::compiler_interface::GotocCodegenBackend::codegen_items` and select "focus on subtree only"... ( you can always go back with the cookie crumbs thing in between the flamegraph and thread view).