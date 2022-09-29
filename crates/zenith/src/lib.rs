pub mod prelude {
    use serde::{Deserialize, Serialize};

    pub enum ZenDeserializer {
        JSON,
        Bincode,
        None,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Input {
        pub name: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Output {
        pub greeting: String,
    }
}
