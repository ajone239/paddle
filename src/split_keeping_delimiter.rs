pub struct SplitKeepingDelimiter<'a, 'b> {
    data: &'a str,
    delimiters: &'b [char],
    start: usize,
}

impl<'a, 'b> Iterator for SplitKeepingDelimiter<'a, 'b> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() == self.start {
            return None;
        }

        if self
            .delimiters
            .contains(&self.data[self.start..].chars().next().unwrap())
        {
            let c = &self.data[self.start..self.start + 1];
            self.start += 1;
            return Some(c);
        }

        let start = self.start;
        let mut end = self.start;

        for c in self.data[start..].chars() {
            if self.delimiters.contains(&c) {
                break;
            }

            end += c.len_utf8();
        }

        self.start = end;

        return Some(&self.data[start..end]);
    }
}

pub trait SplitKeepingDelimiterExt: ::std::ops::Index<::std::ops::RangeFull, Output = str> {
    fn split_keeping_delimiter<'a>(
        &'a self,
        delimiters: &'a [char],
    ) -> SplitKeepingDelimiter<'a, 'a> {
        SplitKeepingDelimiter {
            data: &self[..],
            delimiters: delimiters,
            start: 0,
        }
    }
}

impl SplitKeepingDelimiterExt for str {}

#[cfg(test)]
mod test {
    use super::SplitKeepingDelimiterExt;

    #[test]
    fn split_with_delimiter() {
        let delims = &[',', ';'][..];
        let items: Vec<_> = "alpha,beta;gamma".split_keeping_delimiter(delims).collect();
        assert_eq!(&items, &["alpha", ",", "beta", ";", "gamma"]);
    }

    #[test]
    fn split_with_delimiter_allows_consecutive_delimiters() {
        let delims = &[',', ';'][..];
        let items: Vec<_> = ",;".split_keeping_delimiter(delims).collect();
        assert_eq!(&items, &[",", ";"]);
    }
}
