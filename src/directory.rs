use std::io;
use tokio::fs;

const DIRECTORY_HTML: &str = include_str!("../directory.html");

pub async fn render_directory_template(pre_processed_dir: &str) -> io::Result<String> {
    let mut template = DIRECTORY_HTML.to_string();
    let mut dir = pre_processed_dir.to_string();

    // handling back button
    if pre_processed_dir.contains("index.html") {
        template = template.replace("{{back}}", "");
        dir = pre_processed_dir.replace("index.html", "");
    } else {
        template = template.replace("{{back}}", "<a class='back' href='..'>&larr; Back</a>");
    };

    // replacing title
    template = template.replace("{{title}}", dir.as_str());

    // handling listing
    let mut directories = fs::read_dir(&dir).await?;
    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    loop {
        if let Some(item) = directories.next_entry().await? {
            let file_type = item.file_type().await?;
            if file_type.is_dir() {
                if let Ok(path) = item.path().into_string() {
                    dirs.push(path);
                }
            } else if file_type.is_file() {
                if let Ok(path) = item.path().into_string() {
                    files.push(path);
                }
            }
        } else {
            break;
        }
    }

    let mut item_list = String::new();
    for mut item in dirs {
        item = item.replace(&dir, "");
        item_list.push_str(&format!(
            "<li class='dir'><a href='{item}/'>{item}</a></li>\n"
        ));
    }
    for mut item in files {
        item = item.replace(&dir, "");
        item_list.push_str(&format!(
            "<li class='file'><a href='{item}'>{item}</a></li>\n"
        ));
    }

    template = template.replace("{{dirs}}", item_list.as_str());

    Ok(template)
}
