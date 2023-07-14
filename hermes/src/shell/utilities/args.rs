/// Splits a line into individual parameters based on certain rules.
///
/// The function takes a line as input and returns a vector of strings, where each string represents
/// an individual parameter extracted from the line. The splitting of parameters follows the following rules:
///
/// - Single words are treated as separate parameters.
/// - Words enclosed within double quotation marks or single quotation marks are treated as a single parameter.
///
/// # Parameters
/// - `command`: String that is parse
///
/// # Returns
///
/// A vector of strings representing individual parameters extracted from the line.
///
/// # Regular Expression
///
/// The regular expression used for splitting the line into parameters is as follows:
///
/// ```text
/// [^\s"']+|"([^"]*)"|'([^']*)'
/// ```
///
/// This regular expression consists of three alternatives:
///
/// 1. `[^\s"']+`: Matches a sequence of characters that are not whitespace, double quotation marks, or apostrophes.
/// 2. `"([^"]*)"`: Matches a sequence of characters within double quotation marks and captures the content inside the quotes.
/// 3. `'([^']*)'`: Matches a sequence of characters within apostrophes and captures the content inside the apostrophes.
///
/// By using this regular expression, the function splits the line into individual parameters, taking into account
/// the specified rules for quoted phrases and single words.
///
pub fn split_arguments(command: &str) -> Vec<String> {
    let mut parameters = vec!["hermes".to_string()];

    let re = match regex::Regex::new(r#"[^\s"']+|"([^"]*)"|'([^']*)'"#) {
        Ok(re) => re,
        Err(e) => {
            for line in e.to_string().lines() {
                tracing::error!("{}", line);
            }
            return parameters;
        }
    }; 

    for capture in re.captures_iter(command) {
        if let Some(capture) = capture.get(0) {
            let parameter = capture.as_str().trim_matches('\'').trim_matches('"').to_string();
            parameters.push(parameter);
        }
    }

    parameters
}
