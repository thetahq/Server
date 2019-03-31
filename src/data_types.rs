#[derive(Serialize, Deserialize)]
pub struct TestMessage<'wtf> {
    pub message: &'wtf str
}