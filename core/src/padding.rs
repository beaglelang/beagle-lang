/// A String wrapper that repeats itself count times when converted into a String
pub struct Padding {
    string: String,
    count: usize
}

impl std::fmt::Display for Padding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for _ in 0..self.count {
            write!(f, "{}", self.string)?;
        }

        Ok(())
    }
}

/// Creates padding with the string and count provided
pub fn padding<S: AsRef<str>>(string: S, count: usize) -> Padding {
    Padding {
        string: string.as_ref().to_string(),
        count
    }
}

pub fn padding_until<S: AsRef<str>>(string: S, limit: usize) -> Padding{
    padding(string, limit - 1)
}