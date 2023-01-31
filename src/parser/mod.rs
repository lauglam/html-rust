use std::collections::HashMap;
use crate::dom::{Node, Payload, Tag};

mod input;

pub use input::Input;

/// Parses the tag document and returns a Dom structure tree.
///
/// # Arguments
/// * `doc` - tag document
///
/// # Errors
/// * If the document ends in the middle of a tag or double quote.
///
/// # Examples
/// ```rust
/// let html = r#"
/// <body>
///   <h1 class="h1">Hello</h1>
/// </body>
/// "#;
///
/// if let Ok(node) = html::parse(&html) {
///     println!("{:#?}", node);
/// }
/// ```
///
/// output:
///
/// ```text
/// Node {
///     rc_ref: NodeData {
///         payload: Tag(
///             Tag {
///                 name: "root",
///                 attrs: None,
///                 self_closing: false,
///                 terminator: false,
///             },
///         ),
///         parent: RefCell {
///             value: (Weak),
///         },
///         children: RefCell {
///             value: [
///                 NodeData {
///                     payload: Tag(
///                         Tag {
///                             name: "body",
///                             attrs: None,
///                             self_closing: false,
///                             terminator: false,
///                         },
///                     ),
///                     parent: RefCell {
///                         value: (Weak),
///                     },
///                     children: RefCell {
///                         value: [
///                             NodeData {
///                                 payload: Tag(
///                                     Tag {
///                                         name: "h1",
///                                         attrs: Some(
///                                             {
///                                                 "class": "h1",
///                                             },
///                                         ),
///                                         self_closing: false,
///                                         terminator: false,
///                                     },
///                                 ),
///                                 parent: RefCell {
///                                     value: (Weak),
///                                 },
///                                 children: RefCell {
///                                     value: [
///                                         NodeData {
///                                             payload: Text(
///                                                 "Hello",
///                                             ),
///                                             parent: RefCell {
///                                                 value: (Weak),
///                                             },
///                                             children: RefCell {
///                                                 value: [],
///                                             },
///                                         },
///                                     ],
///                                 },
///                             },
///                         ],
///                     },
///                 },
///             ],
///         },
///     },
/// }
/// ```
pub fn parse(doc: &str) -> Result<Node, String> {
    let mut input = Input::new(doc);
    let mut node_vec = create_node_vec(&mut input)?;
    // debug_print_node_vec(&node_vec);

    let tag = Tag::new("root");
    let payload = Payload::Tag(tag);

    let mut root = Node::new(payload);
    create_node_tree(&mut node_vec, &mut root);

    Ok(root)
}

/// Returns the value of the tag's attribute.
///
/// State to receive:
/// The cursor points to the first '"' or '\''.
/// "<value>"
/// or
/// '<value>'
/// or
/// <value>
fn parse_tag_attr_value(input: &mut Input, tag_end: usize, delimiter: char) -> Result<String, String> {
    if delimiter != ' ' {
        // move cursor to after '"' or '\''
        input.next();
    }

    let value_bgn = input.get_cursor();

    let value_end;
    match input.find(delimiter) {
        Some(cursor) => {
            if cursor < tag_end {
                // value"
                //      ^
                //      this is delimiter
                value_end = cursor;
            } else if delimiter == ' ' {
                // value>
                //      ^
                //      the end of tag
                value_end = tag_end;
            } else {
                return Err(format!(
                    "There is no delimiter({}) to terminate the attribute.",
                    delimiter
                ));
            }
        }
        None => {
            return Err(format!("Input ends in the middle of delimiter({})", delimiter));
        }
    }

    if value_bgn == value_end {
        // value is empty
        return Ok(String::new());
    }

    input.set_cursor(value_end);
    input.get_string(value_bgn, value_end)
}

/// Gets the cursor position at the end of tag.
///
/// <tag attr="value" >
///                   ^
///                   Return this position.
fn get_tag_end(input: &mut Input) -> Result<usize, String> {
    let save_cursor_pos = input.get_cursor();
    let mut res = 0;

    let mut in_double_quote = false;
    while !input.is_end() {
        input.next_char();
        if input.expect('\"') {
            in_double_quote = !in_double_quote;
        }

        if !in_double_quote && input.expect('>') {
            // make sure the '>' is not inside double quotes
            res = input.get_cursor();
            break;
        }
    }

    input.set_cursor(save_cursor_pos);
    match res {
        0 => Err(String::from("Input ends in the middle of the tag.")),
        _ => Ok(res),
    }
}

/// Parses tag attributes.
///
/// State to receive:
/// The cursor points to the first character of <attr>.
/// <attr>[ = "<value>"] [/]>
/// or
/// <attr>[ = '<value>'] [/]>
/// or
/// <attr>[ = <value>] [/]>
fn parse_tag_attr(input: &mut Input, mut tag: Tag) -> Result<Tag, String> {
    // get the end position of the tag
    let tag_end = get_tag_end(input)?;

    let mut attr_map = HashMap::new();

    // get attribute and their value
    // the terminal '/' is also an attribute
    loop {
        if input.expect('>') {
            input.next();
            break;
        }

        // get attribute name
        let attr_name_bgn = input.get_cursor();
        let mut attr_name_end = tag_end;

        // if the tag contains '=', that position is the end position of the attribute name
        //
        // attr="value"
        //     ^
        if let Some(cursor) = input.find('=') {
            if cursor < tag_end {
                attr_name_end = cursor;
            }
        }

        // if the tag contains an ' ' and it precedes '=',
        // make that position the end position of the attribute name
        //
        // attr = "value"
        //     ^
        if let Some(cursor) = input.find(' ') {
            if cursor < attr_name_end {
                attr_name_end = cursor;
            }
        }

        input.set_cursor(attr_name_end);
        let attr_name = input.get_string(attr_name_bgn, attr_name_end)?;

        // get attribute value
        let mut attr_value = String::new();
        if input.get_cursor() != tag_end {
            // if the tag contains an "="
            if let Some(cursor) = input.find('=') {
                if cursor < tag_end {
                    // move cursor to '='
                    input.set_cursor(cursor);
                    // move cursor to after '='
                    input.next_char();
                    if input.expect('"') {
                        // attr = "value"
                        //        ^
                        match parse_tag_attr_value(input, tag_end, '"') {
                            Ok(v) => attr_value = v,
                            Err(e) => return Err(e),
                        }
                    } else if input.expect('\'') {
                        // attr = 'value'
                        //        ^
                        match parse_tag_attr_value(input, tag_end, '\'') {
                            Ok(v) => attr_value = v,
                            Err(e) => return Err(e),
                        }
                    } else {
                        // attr = value
                        //        ^
                        match parse_tag_attr_value(input, tag_end, ' ') {
                            Ok(v) => attr_value = v,
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }

        attr_map.insert(attr_name, attr_value);

        if input.expect('>') {
            // the end of tag
            input.next();
            break;
        }

        // move to the next attribute
        input.next_char();
    }

    // if the attribute contains '/', remove it
    if let Some(_) = attr_map.remove("/") {
        // set the tag is self-closing
        tag.set_self_closing(true);
    }

    tag.set_attrs(attr_map);

    Ok(tag)
}

/// Parses the tag name.
/// the tag name is trimmed.
///
/// State to receive:
/// The cursor points to the first character of <tag_name>.
/// <tag_name> [<attr>[="<value>"]] [/]>
/// or
/// <tag_name> [<attr>[='<value>']] [/]>
fn parse_tag_name(input: &mut Input, terminator: bool) -> Result<Tag, String> {
    // get the start position of the tag name
    let name_bgn = input.get_cursor();

    // get the end position of the tag
    let tag_end = get_tag_end(input)?;

    let mut name_end = tag_end;

    // if the tag contains ' ', make that position the end position of the tag name
    if let Some(cursor) = input.find(' ') {
        // li attr="value"
        //   ^
        if cursor < tag_end {
            name_end = cursor;
        }
    }

    input.set_cursor(name_end);
    let tag_name = input.get_string(name_bgn, name_end)?;
    let tag_name = tag_name.trim();

    let mut tag = Tag::new(tag_name);
    tag.set_terminator(terminator);

    if input.expect('>') {
        // <tag>
        //     ^
        //
        // move cursor to after '>'
        input.next();
        // tag is end, return directly
        return Ok(tag);
    }

    // tag has attributes
    // <tag attr="value">
    //     ^
    input.next_char();

    // if there are spaces before the '>'
    // <tag >
    //      ^
    if input.expect('>') {
        input.next();
        return Ok(tag);
    }

    return parse_tag_attr(input, tag);
}

/// Parses the tag and returns a Node structure.
///
/// State to receive:
/// The cursor points to the first '<'.
/// <[/]<tag_name> [<attr>[="<value>"]] [/]>
/// or
/// <[/]<tag_name> [<attr>[='<value>']] [/]>
fn parse_tag(input: &mut Input) -> Result<Node, String> {
    // move cursor to after '<'
    input.next();

    let mut terminator = false;
    if input.expect('/') {
        // move cursor to after '/'
        input.next();
        terminator = true;
    }

    let tag = parse_tag_name(input, terminator)?;
    let payload = Payload::Tag(tag);
    // TODO debug
    // println!("{:#?}", payload);
    let node = Node::new(payload);

    Ok(node)
}

/// Parses the comment and returns a Node structure.
///
/// State to receive:
/// The cursor points to the first '<'.
/// <!-- <comment> -->
fn parse_comment(input: &mut Input) -> Result<Node, String> {
    // get the position after '<!--'
    let bgn = input.get_cursor() + "<!--".len();
    let end;

    match input.find_str("-->") {
        Some(cursor) => {
            // move cursor to after "-->"
            input.set_cursor(cursor + "-->".len());
            end = cursor;
        }
        None => return Err(String::from("Input ends in the middle of the comment.")),
    }

    let payload = Payload::Comment(input.get_string(bgn, end)?);
    // TODO debug
    // println!("{:#?}", payload);
    let node = Node::new(payload);

    Ok(node)
}

/// Parses the text and returns a Node structure.
///
/// State to receive:
/// <text>
fn parse_text(input: &mut Input) -> Result<Node, String> {
    let bgn = input.get_cursor();

    let end;
    // get the beginning of the next tag as the end of text
    match input.find('<') {
        Some(cursor) => {
            // text <tag ...
            //      ^
            //      the end of text
            input.set_cursor(cursor);
            end = cursor;
        }
        None => {
            input.next_char();
            end = input.get_cursor();
        }
    }

    let payload = Payload::Text(input.get_string(bgn, end)?);
    // TODO debug
    // println!("{:#?}", payload);
    let node = Node::new(payload);

    Ok(node)
}

/// Gets the code of the script tag as text.
fn parse_text_script(input: &mut Input) -> Result<Node, String> {
    let bgn = input.get_cursor();
    let end;

    match input.find_str("</script") {
        Some(cursor) => {
            // </script
            // ^
            // the end of script
            input.set_cursor(cursor);
            end = cursor;
        }
        None => return Err(String::from("Input ends in the middle of the tag.")),
    }

    let payload = Payload::Text(input.get_string(bgn, end)?);
    let node = Node::new(payload);

    Ok(node)
}

/// Parses "<!doctype html>".
/// case insensitive.
///
/// State to receive:
/// The cursor points to the first '<'.
/// <!doctype html>
#[allow(dead_code)]
fn parse_doctype(input: &mut Input) -> Result<Node, String> {
    if !input.expect_str_insensitive("<!doctype html>") {
        return Err(String::from("Input is not html."));
    }

    // Set the tag name to "doctype"
    input.next(); // move cursor to '!'
    input.next(); // move cursor to 'd'
    let bgn = input.get_cursor();
    let end = bgn + "doctype".len();
    input.set_cursor(end); // move cursor to the ' ' before the "html"
    let mut tag = Tag::new(&input.get_string(bgn, end)?);

    input.next(); // move cursor to 'h'

    // Set the attribute to "html"
    // The value of attribute is ""
    let mut attr: HashMap<String, String> = HashMap::new();
    let bgn = input.get_cursor();
    let end = bgn + "html".len();
    input.set_cursor(end); // move cursor to '>'
    attr.insert(input.get_string(bgn, end)?, String::new());
    tag.set_attrs(attr);

    let payload = Payload::Tag(tag);
    let node = Node::new(payload);

    input.next(); // move cursor to after '>'

    Ok(node)
}

/// Parses the tag document and returns the Vec of the Node structure.
fn create_node_vec(input: &mut Input) -> Result<Vec<Node>, String> {
    let mut node_vec = Vec::new();

    // move cursor to the fist '<'
    while !input.expect('<') {
        input.next_char();
    }

    /*
    match parse_doctype(input) {
        Ok(node) => node_vec.push(node),
        Err(e) => return Err(e),
    }
     */

    while !input.is_end() {
        // TODO debug
        // println!("check: {}", input.get_char(input.get_cursor())?);

        if input.expect_str("<!--") {
            // comment
            match parse_comment(input) {
                Ok(node) => node_vec.push(node),
                Err(e) => return Err(e),
            }
        } else if input.expect('<') {
            // tag
            match parse_tag(input) {
                Ok(node) => {
                    // if the node is script tag
                    let mut is_bgn_script = false;
                    if let Payload::Tag(tag) = node.get_payload() {
                        if tag.get_name() == "script" && !tag.is_terminator() {
                            is_bgn_script = true;
                        }
                    }

                    node_vec.push(node);

                    // if the node is script tag and has text
                    if is_bgn_script && !input.expect('<') {
                        match parse_text_script(input) {
                            Ok(node) => node_vec.push(node),
                            Err(e) => return Err(e),
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        } else {
            if input.expect(' ') || input.expect('\n') {
                // skip ' ' and '\n'
                input.next_char();
            }

            if !input.expect('<') {
                // text
                match parse_text(input) {
                    Ok(node) => node_vec.push(node),
                    Err(e) => return Err(e),
                }
            }
        }
    }

    Ok(node_vec)
}

/// Debugging function for node_vec.
#[allow(dead_code)]
fn debug_print_node_vec(node_vec: &Vec<Node>) {
    for node in node_vec {
        match node.get_payload() {
            Payload::Tag(tag) => println!("{:#?}", tag),
            Payload::Text(text) => println!("{:#?}", text),
            Payload::Comment(text) => println!("{:#?}", text),
        }
    }
}

/// Finds the end tag paired with `starter` from node_vec and return its index.
fn find_terminator(node_vec: &mut Vec<Node>, starter: &Tag) -> Option<usize> {
    for i in 0..node_vec.len() {
        let node = node_vec.get(i).unwrap();
        if let Payload::Tag(tag) = node.get_payload() {
            if tag.is_terminator() && starter.get_name() == tag.get_name() {
                return Some(i);
            }
        }
    }

    None
}

/// If the tag is not terminator, add it to the child.
fn create_node_tree(node_vec: &mut Vec<Node>, parent: &Node) {
    while !node_vec.is_empty() {
        let mut node = node_vec.remove(0);

        if let Payload::Tag(tag) = node.get_payload() {
            if tag.is_terminator() {
                // get terminator. `</ tag>`
                return;
            }

            if !tag.is_self_closing() {
                // If not self-closing. not `<tag />`
                if let Some(terminator_idx) = find_terminator(node_vec, tag) {
                    // If there is terminator tag
                    if terminator_idx == 0 {
                        // If there are no children, delete the terminator tag. <tag></ tag>
                        node_vec.remove(0);
                    } else {
                        // If there are children, recurse
                        create_node_tree(node_vec, &mut node);
                    }
                }
            }
        }

        parent.add_child_and_update_parent(&node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() {
        let html = r#"
        <body>
          <h1 class="h1">Hello</h1>
        </body>
        "#;

        match parse(&html) {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn eq_test() {
        let a = r#"
        <head>
          <title>sample</title>
        </head>
        <body>
          <h1>section</h1>
          <ul>
            <li>list1</li>
            <li>list2</li>
          </ul>
        </body>
        "#;
        let a_node = parse(&a).unwrap();

        let b = r#"
        <head>
          <title>sample</title>
        </head>
        <body>
          <h1>section</h1>
          <ul>
            <li>list1</li>
            <li>list2</li>
          </ul>
        </body>
        "#;
        let b_node = parse(&b).unwrap();

        assert_eq!(a_node == b_node, true);
        assert_eq!(a_node != b_node, false);
    }

    #[test]
    fn ne_test() {
        let a = r#"
        <head>
          <title>sample</title>
        </head>
        <body>
          <h1>section</h1>
          <ul>
            <li>list1</li>
            <li>list2</li>
          </ul>
        </body>
        "#;
        let a_dom = parse(&a).unwrap();

        let b = r#"
        <head>
          <title>sample</title>
        </head>
        <body>
          <h1>section</h1>
          <ul>
            <li>list1</li>
            <li>list3</li>
          </ul>
        </body>
        "#;
        let b_dom = parse(&b).unwrap();

        assert_eq!(a_dom == b_dom, false);
        assert_eq!(a_dom != b_dom, true);
    }
}
