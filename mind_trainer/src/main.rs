
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::process::{Command, Stdio};

struct LearningObjectives;

impl LearningObjectives {
    fn new(class: &str) -> Result<Vec<String>, Box<dyn Error>> {

        let file_path = format!("./objectives/{}", class);


        // Create a vector to store the lines
        let mut lines_vector: Vec<String> = Vec::new();

        // Open the file in read-only mode
        if let Ok(file) = File::open(&file_path) {
            // Create a BufReader for efficient line-by-line reading
            let reader = io::BufReader::new(file);

            // Iterate over each line
            for line in reader.lines() {
                // Push each line into the vector
                if let Ok(line_content) = line {
                    lines_vector.push(line_content);
                }
            }
        } else {
            println!("Failed to open the file: {}", file_path);

        }

        Ok(lines_vector)

    }

}


fn genereate_question() -> std::io::Result<()>{
    
    let vec = match LearningObjectives::new("multivariable_calculus") {
        Ok(v) => v,
        Err(_) => {panic!("Error reading the vector!")}
    };
    
    
    let mut rng = thread_rng(); // Create a random number generator
    let random_element = vec.choose(&mut rng).unwrap();
    

    // Define the model and prompt
    let model = "mind_trainer";

    // Spawn the `ollama` command with the model and prompt
    let mut child = Command::new("ollama")
        .arg("run")
        .arg(model)
        .arg(random_element)
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


fn main() {
 
    genereate_question().expect("Failed to generate question");

}