use std::collections::HashMap;
use std::error::Error;
use lopdf::Document;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Sections {
    sections: HashMap<String, Section>,
}

impl Sections {
    pub fn new() -> Self {
        Sections {
            sections: HashMap::new(),
        }
    }

    /// Adds a secition to the outline, by adding
    pub fn add(&mut self, s: Section) -> () {
        // compare to sec
        let pattern = Regex::new(r"^\d+").unwrap();

        if let Some(section_num) = pattern.find(&s.id) {
            let section_num = String::from(section_num.as_str());

            if let Some(parent) = self.sections.get_mut(&section_num) {
                // add the child
                parent.children.push(s);
            } else {
                // add the parent
                self.sections.insert(section_num, s);
            }
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Section {
    id: String,
    title: String,
    page: u8,
    children: Vec<Section>,
}

impl Section {
    pub fn new(id: String, title: String, page: u8) -> Self {
        Section {
            id,
            title,
            page,
            children: Vec::new(),
        }
    }
}

/// NEEDS OPTIMIZATION
/// Gets the table of contents from the pdf in the format reuquired for EN625
///
fn get_contents(doc: &Document) -> Value {
    let contents_pages = get_contents_pages(&doc);
    let text = doc.extract_text(&contents_pages).unwrap_or_default();

    let re = Regex::new(r"(\d+(\.\d+)*)(\s[^\d]+)\s(\d+)").unwrap();

    let mut sections = Sections::new();

    // loop through regex matches
    for mat in re.find_iter(&text) {
        let mut match_str = mat.as_str().to_string();

        if let Some(pos) = match_str.find(' ') {
            // Replace all dots after the first space with a space (clean up)
            match_str.replace_range(pos + 1.., &match_str[pos + 1..].replace('.', ""));
            let mut parts = match_str.split_whitespace().collect::<Vec<&str>>();

            // build Section structs
            let section = parts.remove(0);
            let page = parts.pop().unwrap().parse::<u8>().unwrap();
            let title = parts.join(" ");

            let section = Section::new(section.to_string(), title, page);
            sections.add(section)
        }
    }

    // sort the dict so that the json is in order
    let mut keys: Vec<&String> = sections.sections.keys().collect();
    keys.sort_by(|a, b| a.parse::<i32>().unwrap().cmp(&b.parse::<i32>().unwrap())); // keys are strings which sorts 1, 10, 2,3

    // TODO! optimize the need for this
    let mut sorted_sections = Vec::new();

    for key in keys {
        if let Some(value) = sections.sections.get(key) {
            sorted_sections.push(serde_json::to_value(value).unwrap())
        }
    }

    json!(sorted_sections)
}

/// Takes an EN621 PDF notes for the week and returns the page numbers that the table of contents are on
///  
fn get_contents_pages(doc: &Document) -> Vec<u32> {
    let pages = doc.get_pages();
    let mut contents_pages = Vec::new();
    let mut is_contents = false;

    for (i, _) in pages.iter().enumerate() {
        let page_number = (i + 1) as u32;
        let text = doc.extract_text(&[page_number]).unwrap_or_default();

        // check when contents is over and set flag to false
        // first item of text on the page will start with 1
        if is_contents && text.starts_with("1") {
            break;
        }

        // Find the page number for the contents
        if text.contains("Contents") || is_contents {
            is_contents = true;
            contents_pages.push(page_number);
        }
    }

    return contents_pages;
}


pub fn get_en625_(week: u8) -> Result<Value, Box<dyn Error>> {
    let file = format!("artifacts/algorithms/m{}/m{}.pdf", week, week);

    match Document::load(file) {
        Ok(document) => {
            Ok(get_contents(&document))
        }
        Err(err) => Err(Box::new(err)),
    }
}

pub fn generate_reading_schedule(week: u8) -> std::io::Result<()> {

    // Define the model and prompt
    let model = "schedule_builder";

    let context = get_en625_(week).expect("Could not get pdf!");

    // Spawn the `ollama` command with the model and prompt
    let mut child = Command::new("ollama")
        .arg("run")
        .arg(model)
        .arg(context.to_string())
        .stdout(Stdio::piped())
        .spawn()?;

    // Ensure the child process was successfully spawned
    let stdout = child.stdout.take().expect("Failed to capture stdout");

    // Create a buffered reader for streaming output
    let reader = BufReader::new(stdout);

    // Stream and print each line of output
    for line in reader.lines() {
        match line {
            Ok(content) => println!("{}", content),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    // Wait for the child process to finish
    let status = child.wait()?;
    if !status.success() {
        eprintln!("Ollama command failed with status: {}", status);
    }

    Ok(())


}


mod tests{
    use super::get_en625_;


    #[test]
    fn test_content() {
        let v = get_en625_(3).expect("Failed to get week 3");
        println!("{}", v)
    }
}
