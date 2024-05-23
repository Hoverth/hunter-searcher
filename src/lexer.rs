use log::debug;

// lexer inspired by tsoding's search engine in rust
// {https://youtu.be/hm5xOJiVEeg}

pub struct Lexer<'a> {
    content: &'a [char]
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Lexer { content }
    }

    fn trim_left(&mut self) {
        while !self.content.is_empty() && self.content[0].is_whitespace(){
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    pub fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();

        if self.content.is_empty() {
            return None
        }
        
        // urls
        if self.content[0] == 'h' && self.content.len() > 8 && self.content[0..3] == "http".chars().collect::<Vec<char>>()[..] {
            debug!("Found URL...");
 
            let mut n = 1;
            while n < self.content.len() && !self.content[n].is_whitespace() {
                n += 1;
            }
            return Some(self.chop(n));
        }

        // words
        if self.content[0].is_alphabetic() {
            debug!("Found word...");
            let mut n = 1;
            while n < self.content.len() && self.content[n].is_alphanumeric() {
                n += 1;
            }
            return Some(self.chop(n));
        }

        // numbers, including decimals, dates, time, and prices
        if self.content[0].is_numeric() || self.content[0] == '$' {
            debug!("Found number...");

            let mut n = 1;
            while n < self.content.len() && (
                            self.content[n].is_numeric() ||
                            self.content[n] == ',' ||
                            self.content[n] == '.' ||
                            self.content[n] == '_' ||
                            self.content[n] == '-' ||
                            self.content[n] == '/' ||
                            self.content[n] == ':'
                            ) 
            {
                n += 1;
            }
            return Some(self.chop(n));
        }

        if !self.content[0].is_alphanumeric() {
            debug!("Found symbol...");
            self.chop(1);
            return self.next_token();
        }

        Some(self.chop(1))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
