#[derive(Debug)]
pub struct Examinee {
    pub id: u64,
    pub surname: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub seat_number: u64,
    pub time: String,
    pub room_assignment: u64,
    pub test_center_code: u64,
}

#[derive(Debug)]
pub struct RawQualifier {
    pub id: u64,
    pub surname: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub seat_number: u32,
    pub time: String,
    pub room_assignment: u32,
    pub test_center_code: u32,
    pub test_center_name: String,
    pub test_center_addr: String,
}
