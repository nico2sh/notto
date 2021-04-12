use pulldown_cmark::{Options, Parser};
use super::front_matter::FrontMatter;

#[derive(Debug, Clone)]
pub struct Note {
    pub front_matter: FrontMatter,
    pub content: String
}

impl Default for Note {
    fn default() -> Self {
        Self {
            front_matter: FrontMatter::default(),
            content: String::new()
        }
    }
}

impl Note {
    pub fn new<S: Into<String>>(front_matter: FrontMatter, content: S) -> Self {
        Self { front_matter, content: content.into() }
    }

    pub fn from_text<S>(text: S) -> Note where S: AsRef<str> {
        let lines = text.as_ref().lines();
        let mut front_matter = vec![];
        let mut content = vec![];
        let mut in_front_matter = false;
        lines.enumerate().for_each(|(pos, line)| {
            content.push(line);
            if pos != 0 && line.trim() == "---" && in_front_matter {
                in_front_matter = false;
                content.clear();
            }
            if in_front_matter { front_matter.push(line); }
            if pos == 0 && line.trim() == "---" { in_front_matter = true; }
        });
        // The front matter section hasn't been closed
        if in_front_matter == true { front_matter.clear(); }

        let cont = content.join("\n");

        // Front Matter
        let mut fm = match serde_yaml::from_str(front_matter.join("\n").as_str()) {
            Ok(f) => f,
            Err(_) => FrontMatter::default()
        };
        if fm.title == None {
            let title = extract_title(&cont);
            fm.title = Some(title);
        }

        Note::new(fm, cont)
    }

    pub fn to_text(&self) -> String {
        let mut text = String::new();
        let front_matter_text = match serde_yaml::to_string(&self.front_matter) {
            Ok(fm_text) => fm_text,
            Err(_e) => String::new()
        };
        text.push_str(&front_matter_text);
        text.push_str("---\n");
        text.push_str(&self.content);

        text
    }

    pub fn get_title(&self) -> String {
        match &self.front_matter.title {
            Some(title) => title.clone(),
            None => extract_title(&self.content)
        }
    }
}

fn extract_title<S>(note_text: S) -> String where S: AsRef<str> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    for line in note_text.as_ref().lines() {
        let parser = Parser::new_ext(line, options);
        let mut buffer = String::new();

        for event in parser {
            match event {
                pulldown_cmark::Event::Text(text) => buffer.push_str(&text),
                pulldown_cmark::Event::Code(text) => buffer.push_str(&text),
                _ => {}
            }
        }

        buffer = get_first_sentence(buffer);
        if !buffer.is_empty() {
            return buffer;
        }
    }

    "untitled".to_string()
}

fn get_first_sentence<S>(text: S) -> String where S: AsRef<str> {
    let sentence_separators = [ '.', '!', '?', '(' ];
    let line = text.as_ref();

    let mut start_index = 0;
    for (index, c) in line.chars().enumerate() {
        if !sentence_separators.contains(&c) {
            start_index = index;
            break;
        }
    }

    let end_of_sentence = line[start_index..].find(|ref c| {
        sentence_separators.contains(c)
    });

    if let Some(index) = end_of_sentence {
        line[start_index..index + start_index].trim().to_string()
    } else {
        line.trim().to_string()
    }
}

#[cfg(test)]
mod test {
    use chrono::{NaiveDate, NaiveTime};

    use super::Note;

    #[test]
    fn detects_front_matter() {
        let text =
r#"---
title: test note
date: 2021-03-28
time: 17:08:13
---
```
This is a demo note.
```
With more than one line
Actually three.

And an extra line break.
"#;
        let note = Note::from_text(text);
        assert_eq!(Some("test note".to_string()), note.front_matter.title);
        assert_eq!(NaiveDate::from_ymd(2021, 03, 28), note.front_matter.date);
        assert_eq!(NaiveTime::from_hms(17, 08, 13), note.front_matter.time);
        println!("{}", &note.content);
    }

    #[test]
    fn detects_title() {
        let text =
r#"```
This is a demo note.
```
With more than one line
Actually three.

And an extra line break.
"#;
        let note = Note::from_text(text);
        assert_eq!(Some("This is a demo note".to_string()), note.front_matter.title);
    }

    #[test]
    fn detects_title_with_two_sentences() {
        let text =
r#"```
This is a demo note. With two sentences.
```
With more than one line
Actually three.

And an extra line break.
"#;
        let note = Note::from_text(text);
        assert_eq!(Some("This is a demo note".to_string()), note.front_matter.title);
    }

    #[test]
    fn detects_title_with_starting_periods() {
        let text =
r#"
... This is a demo note. With two sentences. And three starting periods.

With more than one line
Actually three.

And an extra line break.
"#;
        let note = Note::from_text(text);
        assert_eq!(Some("This is a demo note".to_string()), note.front_matter.title);
    }

    #[test]
    fn detects_title_with_starting_invalid_line() {
        let text =
r#"
...???
... This is a demo note. With two sentences. And three starting periods.

With more than one line
Actually three.

And an extra line break.
"#;
        let note = Note::from_text(text);
        assert_eq!(Some("This is a demo note".to_string()), note.front_matter.title);
    }
}