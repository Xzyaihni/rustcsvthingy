pub mod csv_reader
{
    type Answer = Vec<String>;
    type Reply = Vec<Answer>;

    #[derive(PartialEq, Debug)]
    pub struct Answers
    {
        labels: Vec<String>,
        replies: Vec<Reply>
    }

    impl Answers
    {
        pub fn parse(file: &str) -> Result<Self, &'static str>
        {
            let answers = parser::parse(file)?;

            for batch in &answers.replies
            {
                if batch.len()!=answers.labels.len()
                {
                    return Err("replies are not the same size as labels");
                }
            }

            Ok(answers)
        }

        pub fn labels(&self) -> &Vec<String>
        {
            &self.labels
        }

        pub fn replies(&self) -> &Vec<Reply>
        {
            &self.replies
        }

        pub fn reply(&self, index: usize) -> Option<&Vec<Answer>>
        {
            self.replies.get(index)
        }

        pub fn question(&self, name: &str) -> Option<Vec<&str>>
        {
            let index = self.label(|label| {label.contains(name)})?;
            Some(self.collect(index))
        }

        pub fn question_exact(&self, name: &str) -> Option<Vec<&str>>
        {
            let index = self.label(|label| {label==name})?;
            Some(self.collect(index))
        }

        fn collect(&self, index: usize) -> Vec<&str>
        {
            let mut out = vec![&self.labels[index][..]];
            for reply in &self.replies
            {
                for choice in &reply[index]
                {
                    out.push(&choice[..]);
                }
            }

            out
        }

        fn label<F>(&self, mut cmp: F) -> Option<usize>
            where
                F: FnMut(&str) -> bool
        {
            for (index, label) in self.labels.iter().enumerate()
            {
                if cmp(label)
                {
                    return Some(index);
                }
            }

            None
        }
    }

    mod parser
    {
        use std::mem;

        use super::Answers;
        use super::Reply;

        struct State
        {
            options: Vec<String>,
            option: String,
            special: bool,
            text: bool,
            next: bool,
            over: bool
        }

        impl State
        {
            fn new() -> Self
            {
                State{options: Vec::new(), option: String::new(),
                    special: false, text: false, next: false, over: false}
            }

            fn update(&mut self, c: char)
            {
                if self.next
                {
                    self.next = false;
                }

                if self.over
                {
                    self.over = false;
                }

                if self.special
                {
                    self.special = false;
                }

                match c
                {
                    '"' =>
                    {
                        self.text = !self.text;
                        self.special = true;
                    },
                    ',' =>
                    {
                        if !self.text
                        {
                            self.over = true;
                        }
                    },
                    ';' =>
                    {
                        self.next = true;
                        self.special = true;
                    },
                    _ => ()
                }
            }

            fn parse(&mut self, c: char) -> Option<Vec<String>>
            {
                if self.text && !self.special
                {
                    self.option.push(c);
                }

                if self.next || self.over
                {
                    self.options.push(mem::replace(&mut self.option, String::new()));
                }

                if self.over
                {
                    return Some(mem::replace(&mut self.options, Vec::new()));
                }

                None
            }
        }

        pub fn parse(file: &str) -> Result<Answers, &'static str>
        {
            let lines = split_lines(&file);
            let mut lines = lines.iter();

            let labels = parse_line(lines.next().ok_or("first line missing")?)
                .into_iter().flatten().collect();

            let mut replies: Vec<Reply> = Vec::new();
            for line in lines
            {
                replies.push(parse_line(&line));
            }

            Ok(Answers{labels, replies})
        }

        fn parse_line(input: &str) -> Reply
        {
            let mut state = State::new();

            let mut line: Reply = Vec::new();
            for c in input.chars()
            {
                state.update(c);
                if let Some(text) = state.parse(c)
                {
                    line.push(text);
                }
            }

            state.update(',');
            line.push(state.parse(',').expect("always returns string after comma"));

            line
        }

        fn split_lines<'a>(file: &'a str) -> Vec<&'a str>
        {
            let mut text = false;
            let mut last_pushed = 0;

            let mut out = Vec::new();
            for (index, c) in file.bytes().enumerate()
            {
                match c
                {
                    b'"' => text = !text,
                    b'\n' =>
                    {
                        if !text
                        {
                            out.push(&file[last_pushed..index]);
                            last_pushed = index;
                        }
                    },
                    _ => ()
                }
            }
            out.push(&file[last_pushed..]);

            out
        }

        #[cfg(test)]
        mod tests
        {
            use super::super::*;

            #[test]
            fn parse_line()
            {
                let result = parser::parse_line(
                    "\"Thingy 🥺\", \"Dingy 🥺\", \"Test!!ъ\", \"one;two\"");

                assert_eq!(result,
                    vec![
                        vec!["Thingy 🥺"],
                        vec!["Dingy 🥺"],
                        vec!["Test!!ъ"],
                        vec!["one", "two"]
                        ]);
            }

            #[test]
            fn parse_full()
            {
                let result = Answers::parse(
                    "\"q1 🥺\", \"q2 wowie\", \"q3 ok\"
                    \"yea\", \"yea;no\", \"yea\"
                    \"what\", \"sure\", \"mhmm\"");

                assert_eq!(result, Ok(Answers
                {
                    labels: vec![
                        String::from("q1 🥺"), String::from("q2 wowie"), String::from("q3 ok")
                        ],
                    replies: vec![
                        vec![
                            vec![String::from("yea")],
                            vec![String::from("yea"), String::from("no")],
                            vec![String::from("yea")]
                            ],
                        vec![
                            vec![String::from("what")],
                            vec![String::from("sure")],
                            vec![String::from("mhmm")]
                            ]]
                }));
            }
        }
    }
}