pub struct Tokenizer<'a> {
    content: &'a [char]
}

impl<'a> Tokenizer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..]
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[..n];
        self.content = &self.content[n..];

        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where 
        P: FnMut(char) -> bool 
    {
        let mut n = 0; 
        while n < self.content.len() && predicate(self.content[n]) {
            n += 1;
        }

        self.chop(n)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.trim_left();

        if self.content.len() == 0 {
            return None
        }

        if self.content[0].is_alphabetic() { 
            return Some(self.chop_while(|x| !x.is_whitespace()))
        }

        if self.content[0].is_numeric() {
            return Some(self.chop_while(|x| x.is_numeric()))
        }

        Some(self.chop(1))
    }
}

