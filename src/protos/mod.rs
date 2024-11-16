pub mod google {
    pub mod cloud {
        pub mod speech {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/google.cloud.speech.v1.rs"));
            }
        }
    }
}
