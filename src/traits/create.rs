pub struct MissingFields {
    pub fields: Vec<&'static str>,
}

pub trait QBCreatable {
    fn can_create(&self) -> Result<(), MissingFields>;
}
