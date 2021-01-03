use errno::errno;
use std::fmt;

#[derive(Debug)]
pub struct IrisErrorWrapper
{
    desc: String,
    error_code: i32,
}

impl IrisErrorWrapper
{
    pub fn new(desc: &str) -> Self
    {
        let e_code = errno();

        IrisErrorWrapper { desc: format!("{}: {}", desc, e_code), error_code: e_code.0 }
    }

    pub fn new_with_code(desc: &str, e_code: i32) -> Self
    {
        IrisErrorWrapper { desc: format!("{}: {}", desc, e_code), error_code: e_code }
    }

    pub fn error_code(&self) -> i32
    {
        self.error_code
    }

    pub fn desc(&self) -> &str
    {
        &self.desc
    }
}

impl fmt::Display for IrisErrorWrapper
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "({}, {})", self.error_code, self.desc)
    }
}