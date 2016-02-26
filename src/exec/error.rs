#[derive(Debug)]
pub enum ExecErrorType {
}

#[derive(Debug)]
pub struct ExecError {
    error_type : ExecErrorType,
    error_msg : String,
}
