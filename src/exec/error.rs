#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExecErrorType {
    PrimaryKeyExist,
}

#[derive(Debug, Clone)]
pub struct ExecError {
    pub error_type : ExecErrorType,
    pub error_msg : String,
}
