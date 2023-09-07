pub mod protot {
    pub mod core {
        tonic::include_proto!("sylklabs.core");
    }
    pub mod scheduler {
        pub mod v1 {
            tonic::include_proto!("sylklabs.scheduler.v1");
        }
    }
}
