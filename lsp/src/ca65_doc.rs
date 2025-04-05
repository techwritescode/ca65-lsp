use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::sync::OnceLock;
use tower_lsp_server::lsp_types::MessageType;
use crate::asm_server::Asm;
use parser::stream::Stream;

pub static CA65_KEYWORDS_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

pub async fn parse_ca65_html(asm_server: &Asm) {
    let arc_config = asm_server.get_configuration();
    let cc65_path = arc_config.as_ref().toolchain.cc65.as_ref().unwrap();
    let ca65_html_path = format!("{cc65_path}\\html\\ca65.html");

    // get contents of ca65.html
    let mut ca65_html_contents = String::new();
    let mut f = std::fs::File::open(ca65_html_path).expect("could not open ca65.html");
    f.read_to_string(&mut ca65_html_contents).expect("could not read ca65.html to string");

    // parse ca65.html
    let ca65_html_stream = Stream::new(ca65_html_contents);
    let mut ca65_html_parser = Ca65HtmlParser::new(ca65_html_stream);
    let hm = ca65_html_parser.parse_to_hashmap();

    CA65_KEYWORDS_MAP.set(hm).unwrap();
}

struct Ca65HtmlParser {
    input: Stream,
    start: usize,
    output: String,
    element_stack: Vec<String>,
    curr_key: String,
    curr_description: String,
    curr_href: String,
}

impl Ca65HtmlParser {
    fn new(input: Stream) -> Self {
        Self {
            input,
            start: 0,
            output: String::from(""),
            element_stack: Vec::new(),
            curr_key: String::from(""),
            curr_description: String::from(""),
            curr_href: String::from(""),
        }
    }
    fn parse_to_hashmap(&mut self) -> HashMap<String, String> {
        let mut hm = HashMap::<String, String>::new();
        while !self.input.at_end() {
            if let Some(c) = self.input.advance() {
                if c != '<' {
                    if !self.curr_key.is_empty() {
                        if self.is_top_element("code") {
                            self.curr_description.push(c);
                        } else if !self.element_stack.contains("h2".as_ref()) {
                            if self.is_top_element("p")
                                || self.is_top_element("a")
                                || self.is_top_element("blockquote")
                                || self.is_top_element("pre")
                            {
                                self.curr_description.push(c);
                            }
                        }
                    }
                    continue
                }
                let mut is_closing_el = false;
                if self.input.peek() == Some('/') {
                    is_closing_el = true;
                    self.input.advance().expect("Unable to advance the stream in parse_to_hashmap");
                }
                self.start = self.input.pos();
                loop {
                    let cc = self.input.peek().expect("Unable to peek next character of stream in parse_to_hashamp");
                    if cc == ' ' || cc == '>' { break }
                }
                let element_name = self.current_string().to_lowercase();
                if is_closing_el {
                    if let Some(el) = self.element_stack.pop() {
                        if el != element_name {
                            // throw warning: closing tag does not match opening tag.
                        }
                        if !self.curr_key.is_empty() {
                            if &el == "h2" {
                                self.curr_description.push_str(" ayyy jimbo\n---\n");
                            } else if &el == "code" && !self.element_stack.contains("blockquote".as_ref()) {
                                self.curr_description.push('`');
                            } else if &el == "blockquote" {
                                self.curr_description.push_str("```\n");
                            } else if &el == "a" && !self.element_stack.contains("h2".as_ref()) {
                                self.curr_description.push_str(&format!("]({})", self.curr_href));
                            } else if &el == "p" {
                                self.curr_description.push_str("\n\n");
                            }
                        }
                    }
                } else {
                    // make sure is not a self-closing html tag
                    if !Vec::<String>::from([
                        "!doctype",
                        "link",
                        "rel",
                        "br",
                        "hr"
                    ]).contains(&element_name) {
                        self.element_stack.push(element_name.clone());
                    }
                }
                if self.input.advance().is_some_and(|c_after_element_name| c_after_element_name == ' ') {
                    if element_name == "a" {
                        self.start = self.input.pos();
                        while self.input.advance().is_some_and(|cc| cc != '"') {}
                        if self.is_top_element("h2") {
                            let cc = self.input.peek().expect("Unable to peek next character of stream looking for keyword");
                            if cc == '.' && self.current_string() == "NAME=\"" {
                                self.start = self.input.pos();
                                while !self.input.match_char('"') {}
                                self.curr_key = self.current_string().clone();
                            }
                        } else if !self.curr_key.is_empty()
                            && !self.element_stack.contains("h2".as_ref())
                            && self.current_string() == "HREF=\""
                        {

                        }
                    }
                }
            }
        }
        hm
    }

    fn is_top_element(&self, el: &str) -> bool {
        self.element_stack.last() == Some(el.as_ref())
    }
    fn current_string(&self) -> String {
        String::from_utf8(self.input[self.start..self.input.pos()].to_vec()).expect("Failed to get string slice from stream")
    }
}