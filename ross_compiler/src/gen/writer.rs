use std::fmt::Write;

pub struct Writer {
    indention: Box<str>,
    current_indention: String,
    buffer: String,
    current: String,
}

impl Writer {
    pub fn new<T: Into<Box<str>>>(indention: T) -> Self {
        Self {
            indention: indention.into(),
            current_indention: String::new(),
            buffer: String::with_capacity(10 * 1024),
            current: String::with_capacity(256),
        }
    }

    #[inline]
    pub fn indent(&mut self) {
        self.current_indention.push_str(&self.indention);
        if self.current.trim_start().is_empty() {
            self.current.clear();
            self.insert_indention();
        }
    }

    #[inline]
    pub fn dedent(&mut self) {
        let len = self.current_indention.len();
        let ind = self.indention.len();
        self.current_indention.truncate(len - ind);
        if self.current.trim_start().is_empty() {
            self.current.clear();
            self.insert_indention();
        }
    }

    fn insert_indention(&mut self) {
        self.current.push_str(&self.current_indention);
    }

    pub fn write<'a, T: Into<&'a str>>(&mut self, s: T) {
        for (i, line) in s.into().split('\n').enumerate() {
            if i > 0 {
                let current = self.current.trim_end();
                if !current.is_empty() {
                    self.buffer.push_str(&current);
                }
                self.buffer.push('\n');
                //
                self.current.clear();
                self.insert_indention();
            }
            self.current.push_str(line);
        }
    }

    #[inline]
    pub fn result(mut self) -> String {
        if !self.current.is_empty() {
            self.buffer.push_str(&self.current);
        }
        self.buffer
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.write(s);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut w = Writer::new("  ");
        w.write("Hello\nWorld");
        w.write("123");
        w.write("4\n5");
        assert_eq!(w.result(), "Hello\nWorld1234\n5")
    }

    #[test]
    fn indention() {
        let mut w = Writer::new("\t");
        w.write("object {\n");
        w.indent();
        w.write("line 1\n");
        w.dedent();
        w.write("}");
        assert_eq!(w.result(), "object {\n\tline 1\n}");
    }

    #[test]
    fn indention_empty_line() {
        let mut w = Writer::new("\t");
        w.write("object {\n");
        w.indent();
        w.write("line 1\n\nS");
        w.dedent();
        w.write("}");
        assert_eq!(w.result(), "object {\n\tline 1\n\n\tS}");
    }
}
