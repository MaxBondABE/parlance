pub trait TransformContent {
    type Transformed;

    fn to_content(&self, content: String) -> Self::Transformed;
    fn append_content<T: AsRef<str>>(&self, content: T) -> Self::Transformed;
    fn to_uppercase(&self) -> Self::Transformed
    where
        Self: AsRef<str>,
    {
        self.to_content(self.as_ref().to_uppercase())
    }
    fn to_lowercase(&self) -> Self::Transformed
    where
        Self: AsRef<str>,
    {
        self.to_content(self.as_ref().to_lowercase())
    }
}

impl TransformContent for &str {
    type Transformed = String;

    fn to_content(&self, content: String) -> Self::Transformed {
        content
    }
    fn append_content<T: AsRef<str>>(&self, content: T) -> Self::Transformed {
        let mut output = String::with_capacity(self.len() + content.as_ref().len());
        output.push_str(self);
        output.push_str(content.as_ref());
        output
    }
}
impl TransformContent for String {
    type Transformed = Self;

    fn to_content(&self, content: String) -> Self::Transformed {
        content
    }
    fn append_content<T: AsRef<str>>(&self, content: T) -> Self::Transformed {
        let mut output = String::with_capacity(self.len() + content.as_ref().len());
        output.push_str(self.as_str());
        output.push_str(content.as_ref());
        output
    }
}
