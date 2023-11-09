use std::{collections::HashMap, fs::File, io::Write, process::exit};

fn main() {
    let mut args = std::env::args();
    if args.len() < 2 {
        println!("Usage: {} <file>", args.next().unwrap());
    } else {
        let file = args.nth(1).unwrap();
        let contents = std::fs::read_to_string(file.clone()).unwrap();
        run(contents, &file);
    }
}

#[derive(Debug, PartialEq)]
enum ListLevels {
    None,
    Open,
}

fn run(content: String, fname: &String) {
    let mut defined_consts: HashMap<String, String> = HashMap::new();
    let mut inbrackets = false;
    let mut linenumber = 0;
    let mut in_listlevel: ListLevels = ListLevels::None;
    let mut rawtml: String = "
        <!DOCTYPE html>
        <html lang=\"en\">
        <head>
            <meta charset=\"UTF-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
            <title>%title%</title>
            <style>
            %theme%
            </style>
        </head>
        <body>
%content%
        </body>
        </html>
    "
    .to_string();

    let mut themes = HashMap::new();
    themes.insert(
        "light",
        "body {
    background-color: #f2f2f2;
    font-family: Arial, sans-serif;
}
li {
    margin-bottom: 10px;
}
a {
    color: black;
    text-decoration: underline;
}
",
    );
    themes.insert(
        "dark",
        "body {
    background-color: #1d1d1d;
    color: #f2f2f2;
    font-family: Arial, sans-serif;
}
li {
    margin-bottom: 10px;
}
a {
    color: #f2f2f2;
    text-decoration: underline;
}
",
    );

    let mut elems: Vec<String> = Vec::new();
    for line in content.lines() {
        linenumber += 1;
        let line = line.trim();
        match line {
            line if line.starts_with("def") => {
                if inbrackets {
                    error(
                        fname.to_string(),
                        linenumber,
                        "You can't define a constant inside the text area".to_string(),
                    );
                }
                let varname = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .trim_end_matches(":");
                let value = line
                    .split_whitespace()
                    .skip(2)
                    .collect::<Vec<&str>>()
                    .join(" ");
                defined_consts.insert(varname.to_string(), value.to_string());
            }
            "{" => {
                if inbrackets {
                    error(
                        fname.to_string(),
                        linenumber,
                        "Syntax error (Brackets already open)".to_string(),
                    );
                } else {
                    inbrackets = true;
                }
            }
            "}" => {
                if !inbrackets {
                    error(
                        fname.to_string(),
                        linenumber,
                        "Syntax error (No brackets to close)".to_string(),
                    );
                } else {
                    inbrackets = false;
                }
            }
            _ if !inbrackets => {
                error(
                    fname.to_string(),
                    linenumber,
                    "You can't define the text outside the text area".to_string(),
                );
            }
            "[" => {
                if in_listlevel != ListLevels::None {
                    error(
                        fname.to_string(),
                        linenumber,
                        "You can't nest lists".to_string(),
                    )
                }
                in_listlevel = ListLevels::Open;
                elems.push("<ul>".to_string());
            }
            "]" => {
                if in_listlevel != ListLevels::Open {
                    error(
                        fname.to_string(),
                        linenumber,
                        "You can't close a list that isn't open".to_string(),
                    )
                }
                in_listlevel = ListLevels::None;
                elems.push("</ul>".to_string());
            }
            line if line.starts_with("-") && in_listlevel == ListLevels::Open => {
                let li = line
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ");
                elems.push(format!("<li>{}</li>", eval(li, &defined_consts)));
            }
            line if line.starts_with("header") => {
                let header = line
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ");
                elems.push(format!("<h1>{}</h1>", eval(header, &defined_consts)));
            }
            line if line.starts_with("paragraph") => {
                let paragraph = line
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ");
                elems.push(format!("<p>{}</p>", eval(paragraph, &defined_consts)));
            }
            line if line.starts_with("img") => {
                let img_full = line
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ");
                let evaled_img = eval(img_full, &defined_consts);
                let parts: Vec<&str> = evaled_img.split('|').collect();
                if parts.len() != 2 {
                    error(
                        fname.to_string(),
                        linenumber,
                        "You must specify an image and its alt text".to_string(),
                    )
                }
                let img = parts[0].trim();
                let alt = parts[1].trim();
                elems.push(format!("<img src=\"{}\" alt=\"{}\">\n<br>", img, alt));
            }
            line if line.starts_with("link") => {
                let link = line
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ");
                let evaled_link = eval(link, &defined_consts);
                let parts: Vec<&str> = evaled_link.split('|').collect();
                if parts.len() != 2 {
                    error(
                        fname.to_string(),
                        linenumber,
                        "You must specify a link and its text".to_string(),
                    )
                }
                let link = parts[0].trim();
                let text = parts[1].trim();
                elems.push(format!("<a href=\"{}\">{}</a>", link, text));
            }
            line if line.starts_with("list") => {
                // The list statement just makes the ezsite code look nicer
                continue;
            }
            _ => {
                error(fname.to_string(), linenumber, "Syntax error".to_string());
            }
        }
    }
    if inbrackets {
        error(
            fname.to_string(),
            linenumber,
            "Unmatched brackets at EOF".to_string(),
        );
    }
    if in_listlevel != ListLevels::None {
        error(
            fname.to_string(),
            linenumber,
            "Unmatched list at EOF".to_string(),
        );
    }
    if defined_consts.contains_key("title") {
        rawtml = rawtml.replace("%title%", &defined_consts["title"]);
    }
    if defined_consts.contains_key("theme") {
        let theme = defined_consts["theme"].clone();
        if themes.contains_key(&theme.as_str()) {
            rawtml = rawtml.replace("%theme%", themes[&defined_consts["theme"].as_str()]);
        } else {
            error(fname.to_string(), linenumber, "Theme not found".to_string());
        }
    } else {
        rawtml = rawtml.replace("%theme%", "");
    }
    rawtml = rawtml.replace("%content%", &elems.join("\n"));
    std::fs::create_dir("made");
    let out = File::create(format!("made/{}.html", fname));
    match out {
        Ok(mut f) => {
            f.write_all(rawtml.as_bytes()).unwrap();
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn eval(input: String, defined_constants: &HashMap<String, String>) -> String {
    let mut output = input.clone();
    for (key, value) in defined_constants {
        output = output.replace(&format!("%{}%", key), value);
    }
    output
}

fn error(filename: String, linenumber: usize, message: String) {
    println!("{}:{}: {}", filename, linenumber, message);
    exit(1);
}
