pub(crate) fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(
            &[
                "./protos/protot/core/task.proto",
                "./protos/protot/core/configs.proto",
                "./protos/protot/scheduler/v1/scheduler.proto",
                "./protos/protot/scheduler/v1/scheduler_worker.proto",
            ],
            &["./protos"],
        )?;
    // prost_build::compile_protos(
    //     &[
    //         "protos/sylklabs/core/configs.proto",
    //         "protos/sylklabs/core/task.proto",
    //         "protos/sylklabs/scheduler/v1/scheduler.proto",
    //         "protos/sylklabs/scheduler/v1/scheduler_worker.proto",
    //     ],
    //     &["protos/"],
    // )?;
    Ok(())
}
