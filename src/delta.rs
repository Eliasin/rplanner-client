use serde_json::Map;
use std::error::Error;

#[derive(Debug)]
pub struct DeltaParseError {
    what: String,
    source: Option<Box<dyn Error>>,
}

impl DeltaParseError {
    fn with_msg(msg: String) -> Self {
        DeltaParseError {
            what: msg,
            source: None,
        }
    }

    pub fn no_ops_field() -> Self {
        Self::with_msg("No ops field found in delta JSON root".to_string())
    }

    pub fn malformed_root_structure(json: serde_json::Value) -> Self {
        Self::with_msg(format!(
            "JSON root or ops has malformed structure: {}",
            json
        ))
    }

    pub fn no_known_operation<O: std::fmt::Display>(operation: O) -> Self {
        Self::with_msg(format!(
            "Unknown op name found while parsing delta: {}",
            operation
        ))
    }

    pub fn malformed_operation<O: std::fmt::Display>(operation: O) -> Self {
        Self::with_msg(format!(
            "Malformed op found while parsing delta: {}",
            operation
        ))
    }

    pub fn incorrect_insert_op_type<O: std::fmt::Display>(operation: O) -> Self {
        Self::with_msg(format!(
            "insert operation has incorrect type, it must be either a string for text or an object for an image or video: {}",
            operation
        ))
    }

    pub fn malformed_insert_attributes<A: std::fmt::Display>(attributes: A) -> Self {
        Self::with_msg(format!(
            "insert operation has incorrect attributes type, it must be either an array: {}",
            attributes
        ))
    }

    pub fn invalid_bold_attribute_type<A: std::fmt::Display>(bold: A) -> Self {
        Self::with_msg(format!("bold attribute must have type of bool: {}", bold))
    }

    pub fn invalid_italic_attribute_type<A: std::fmt::Display>(italic: A) -> Self {
        Self::with_msg(format!(
            "italic attribute must have type of bool: {}",
            italic
        ))
    }

    pub fn invalid_code_attribute_type<A: std::fmt::Display>(code: A) -> Self {
        Self::with_msg(format!("code attribute must have type of bool: {}", code))
    }

    pub fn invalid_header_attribute_type<A: std::fmt::Display>(header: A) -> Self {
        Self::with_msg(format!(
            "header attribute must have type of positive integer: {}",
            header
        ))
    }
}

impl Error for DeltaParseError {}

impl From<serde_json::Error> for DeltaParseError {
    fn from(err: serde_json::Error) -> Self {
        DeltaParseError {
            what: format!("Error while parsing delta from string: {}", err.to_string()),
            source: Some(Box::new(err)),
        }
    }
}

impl std::fmt::Display for DeltaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.what)
    }
}

fn transform_text_with_attribute(
    attributes: &Map<String, serde_json::Value>,
    text: &mut String,
) -> Result<(), DeltaParseError> {
    if let Some(code_block) = attributes.get("code-block") {
        use serde_json::Value::*;
        match code_block {
            Bool(b) => {
                if *b {
                    *text = format!("```\n{}```", text);
                    return Ok(());
                }
            }
            _ => return Err(DeltaParseError::invalid_code_attribute_type(code_block)),
        }
    }

    if let Some(bold) = attributes.get("bold") {
        use serde_json::Value::*;
        match bold {
            Bool(b) => {
                if *b {
                    *text = format!("**{}**", text);
                }
            }
            _ => return Err(DeltaParseError::invalid_bold_attribute_type(bold)),
        }
    }

    if let Some(italic) = attributes.get("italic") {
        use serde_json::Value::*;
        match italic {
            Bool(b) => {
                if *b {
                    *text = format!("*{}*", text);
                }
            }
            _ => return Err(DeltaParseError::invalid_italic_attribute_type(italic)),
        }
    }

    if let Some(header) = attributes.get("header") {
        use serde_json::Value::*;
        match header {
            Number(n) => {
                let n = n
                    .as_u64()
                    .ok_or(DeltaParseError::invalid_header_attribute_type(header))?;
                if n < 5 {
                    *text = format!("{} {}", "#".repeat(n as usize), text);
                }
            }
            _ => return Err(DeltaParseError::invalid_header_attribute_type(header)),
        }
    }

    Ok(())
}

/* The attributes transformation also may need the previous line of the text as delta
 * transformations on newlines apply to the whole previous line.
*/
fn handle_attributes_transformation(
    attributes: &Map<String, serde_json::Value>,
    new_text: &mut String,
    markdown_string: &mut String,
) -> Result<(), DeltaParseError> {
    if new_text == "\n" {
        let mut markdown_lines = markdown_string.split('\n');
        let mut mutation_text = match markdown_lines.next_back() {
            Some(v) => v.to_string(),
            None => {
                // No last line to format
                return Ok(());
            }
        };
        *markdown_string = markdown_lines.collect::<Vec<&str>>().join("\n");
        mutation_text += "\n";

        transform_text_with_attribute(attributes, &mut mutation_text)?;

        *markdown_string += &mutation_text;

        // We already added the newline back into the string before formatting
        // so we do not add it to the end now
        new_text.clear();
    } else {
        transform_text_with_attribute(attributes, new_text)?;
    }

    Ok(())
}

fn handle_delta_insert(
    insert_op: &serde_json::Value,
    attributes: Option<&Map<String, serde_json::Value>>,
    markdown_string: &mut String,
) -> Result<(), DeltaParseError> {
    use serde_json::Value;

    let mut new_text = String::new();

    match insert_op {
        Value::String(s) => new_text += s,
        Value::Object(v) => match v.get("image") {
            Some(v) => {
                unimplemented!()
            }
            None => {}
        },
        _ => return Err(DeltaParseError::incorrect_insert_op_type(insert_op)),
    };

    if let Some(attributes) = attributes {
        handle_attributes_transformation(attributes, &mut new_text, markdown_string)?;
    };

    *markdown_string += &new_text;

    Ok(())
}

fn handle_delta_op(
    op: &serde_json::Map<String, serde_json::Value>,
    markdown_string: &mut String,
) -> Result<(), DeltaParseError> {
    if let Some(v) = op.get("insert") {
        let attributes = match op.get("attributes") {
            Some(att) => match att {
                serde_json::Value::Object(v) => Some(v),
                _ => return Err(DeltaParseError::malformed_insert_attributes(att)),
            },
            None => None,
        };

        handle_delta_insert(v, attributes, markdown_string)?;
    }

    Ok(())
}

fn parse_delta_ops(ops: &Vec<serde_json::Value>) -> Result<String, DeltaParseError> {
    let mut markdown_string = String::new();

    {
        use serde_json::Value::*;
        for op in ops {
            if let Object(v) = op {
                handle_delta_op(v, &mut markdown_string)?;
            } else {
                return Err(DeltaParseError::malformed_operation(op));
            }
        }
    }

    Ok(markdown_string)
}

pub fn parse_delta_to_markdown<S: AsRef<str>>(delta_str: S) -> Result<String, DeltaParseError> {
    let delta_json: serde_json::Value = serde_json::from_str(delta_str.as_ref())?;

    {
        use serde_json::Value::*;
        if let Object(ref v) = delta_json {
            match v.get("ops") {
                Some(v) => {
                    if let Array(v) = v {
                        parse_delta_ops(v)
                    } else {
                        Err(DeltaParseError::malformed_root_structure(delta_json))
                    }
                }
                None => Err(DeltaParseError::malformed_root_structure(delta_json)),
            }
        } else {
            Err(DeltaParseError::no_ops_field())
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_basic_text_parse() -> Result<(), DeltaParseError> {
        let basic = "{ \"ops\": [{ \"insert\": \"testtext\\n\" }] }";
        let multiple_insert =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\" }, { \"insert\": \"nextline\\n\" }] }";

        assert_eq!(parse_delta_to_markdown(basic)?, String::from("testtext\n"));
        assert_eq!(
            parse_delta_to_markdown(multiple_insert)?,
            String::from("testtext\nnextline\n")
        );

        Ok(())
    }

    #[test]
    fn test_parse_text_with_attributes() -> Result<(), DeltaParseError> {
        let bold =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\", \"attributes\": { \"bold\": true } }] }";

        let italic =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\", \"attributes\": { \"italic\": true } }] }";

        let bold_and_italic =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\", \"attributes\": { \"bold\": true, \"italic\": true } }] }";

        let headers_one =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\", \"attributes\": { \"header\": 1 } }] }";

        let headers_two =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\", \"attributes\": { \"header\": 2 } }] }";

        assert_eq!(
            parse_delta_to_markdown(bold)?,
            String::from("**testtext\n**")
        );
        assert_eq!(
            parse_delta_to_markdown(italic)?,
            String::from("*testtext\n*")
        );
        assert_eq!(
            parse_delta_to_markdown(bold_and_italic)?,
            String::from("***testtext\n***")
        );
        assert_eq!(
            parse_delta_to_markdown(headers_one)?,
            String::from("# testtext\n")
        );
        assert_eq!(
            parse_delta_to_markdown(headers_two)?,
            String::from("## testtext\n")
        );
        Ok(())
    }

    #[test]
    fn test_line_formatting() -> Result<(), DeltaParseError> {
        let bold = "{ \"ops\": [{ \"insert\": \"testtext\"}, \
                    { \"insert\": \"\\n\", \"attributes\": { \"bold\": true }} ] }";

        let italic = "{ \"ops\": [{ \"insert\": \"testtext\"}, \
                      { \"insert\": \"\\n\", \"attributes\": { \"italic\": true }} ] }";

        let bold_and_italic = "{ \"ops\": [{ \"insert\": \"testtext\"}, \
                      { \"insert\": \"\\n\", \"attributes\": { \"italic\": true, \"bold\": true }} ] }";

        let headers_one = "{ \"ops\": [{ \"insert\": \"testtext\"}, \
                           { \"insert\": \"\\n\", \"attributes\": { \"header\": 1 }} ] }";

        let headers_two = "{ \"ops\": [{ \"insert\": \"testtext\"}, \
                           { \"insert\": \"\\n\", \"attributes\": { \"header\": 2 }} ] }";

        assert_eq!(
            parse_delta_to_markdown(bold)?,
            String::from("**testtext\n**")
        );
        assert_eq!(
            parse_delta_to_markdown(italic)?,
            String::from("*testtext\n*")
        );
        assert_eq!(
            parse_delta_to_markdown(bold_and_italic)?,
            String::from("***testtext\n***")
        );
        assert_eq!(
            parse_delta_to_markdown(headers_one)?,
            String::from("# testtext\n")
        );
        assert_eq!(
            parse_delta_to_markdown(headers_two)?,
            String::from("## testtext\n")
        );

        Ok(())
    }

    #[test]
    fn test_bold_italics_with_header() -> Result<(), DeltaParseError> {
        let bold = "{ \"ops\": [{ \"insert\": \"testtext\"}, \
                    { \"insert\": \"\\n\", \"attributes\": { \"bold\": true, \"header\": 1 }} ] }";

        let italic = "{ \"ops\": [{ \"insert\": \"testtext\"}, \
                      { \"insert\": \"\\n\", \"attributes\": { \"italic\": true, \"header\": 1 }} ] }";

        assert_eq!(
            parse_delta_to_markdown(bold)?,
            String::from("# **testtext\n**")
        );
        assert_eq!(
            parse_delta_to_markdown(italic)?,
            String::from("# *testtext\n*")
        );

        Ok(())
    }

    #[test]
    fn test_code_block() -> Result<(), DeltaParseError> {
        let code =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\", \"attributes\": { \"code-block\": true } }] }";

        let code_override =
            "{ \"ops\": [{ \"insert\": \"testtext\\n\", \"attributes\": { \"code-block\": true, \"bold\": true } }] }";

        assert_eq!(
            parse_delta_to_markdown(code)?,
            String::from("```\ntesttext\n```")
        );
        assert_eq!(
            parse_delta_to_markdown(code_override)?,
            String::from("```\ntesttext\n```")
        );
        Ok(())
    }
}
